use super::{PagerDocument, PagerLineNumberMode, PagerLineNumberViews};
use anyhow::Result;
use minus::{Pager, hooks::Hook};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

const WARMUP_DELAY: Duration = Duration::from_secs(5);

pub(super) struct ViewWarmup {
    handle: WarmupHandle,
}

#[derive(Clone)]
pub(super) struct WarmupHandle {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    started: bool,
    stopped: bool,
    revision: u64,
    pending: bool,
    request: Option<Request>,
}

struct Request {
    views: PagerLineNumberViews,
    deadline: Instant,
}

impl ViewWarmup {
    pub(super) fn install(pager: &Pager, document: &RwLock<PagerDocument>) -> Result<Self> {
        let warmup = Self::new()?;
        document
            .write()
            .map_err(|_| anyhow::anyhow!("Pager document lock poisoned"))?
            .attach_warmup(warmup.handle.clone());
        let start = warmup.handle.clone();
        pager.add_hook(Hook::PostPagerStart, 0, Box::new(move |_| start.start()))?;
        let stop = warmup.handle.clone();
        pager.add_hook(Hook::PrePagerExit, 0, Box::new(move |_| stop.stop()))?;
        Ok(warmup)
    }

    fn new() -> Result<Self> {
        let handle = WarmupHandle {
            shared: Arc::new(Shared {
                state: Mutex::new(State::default()),
                changed: Condvar::new(),
            }),
        };
        let worker = handle.clone();
        thread::Builder::new()
            .name("mdv-pager-warmup".into())
            .spawn(move || worker.run())?;
        Ok(Self { handle })
    }
}

impl Drop for ViewWarmup {
    fn drop(&mut self) {
        self.handle.stop();
    }
}

impl WarmupHandle {
    pub(super) fn schedule(&self, views: PagerLineNumberViews) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("pager warmup lock poisoned");
        if state.stopped
            || state
                .request
                .as_ref()
                .is_some_and(|job| job.views.shares_cache_with(&views))
        {
            return;
        }
        state.revision = state.revision.wrapping_add(1);
        state.pending = true;
        state.request = Some(Request {
            views,
            deadline: Instant::now() + WARMUP_DELAY,
        });
        self.shared.changed.notify_one();
    }

    pub(super) fn reset(&self) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("pager warmup lock poisoned");
        state.revision = state.revision.wrapping_add(1);
        state.request = None;
        state.pending = false;
        self.shared.changed.notify_one();
    }

    fn start(&self) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("pager warmup lock poisoned");
        if state.started || state.stopped {
            return;
        }
        state.started = true;
        if let Some(request) = &mut state.request {
            request.deadline = Instant::now() + WARMUP_DELAY;
        }
        self.shared.changed.notify_one();
    }

    fn stop(&self) {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("pager warmup lock poisoned");
        state.stopped = true;
        state.request = None;
        state.pending = false;
        self.shared.changed.notify_one();
    }

    fn is_current(&self, revision: u64) -> bool {
        let state = self
            .shared
            .state
            .lock()
            .expect("pager warmup lock poisoned");
        !state.stopped && state.revision == revision
    }

    fn next_request(&self) -> Option<(PagerLineNumberViews, u64)> {
        let mut state = self
            .shared
            .state
            .lock()
            .expect("pager warmup lock poisoned");
        loop {
            if state.stopped {
                return None;
            }
            if state.started && state.pending {
                let request = state
                    .request
                    .as_ref()
                    .expect("pending warmup requires a view");
                let remaining = request.deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    let views = request.views.clone();
                    state.pending = false;
                    return Some((views, state.revision));
                }
                state = self
                    .shared
                    .changed
                    .wait_timeout(state, remaining)
                    .expect("pager warmup lock poisoned")
                    .0;
            } else {
                state = self
                    .shared
                    .changed
                    .wait(state)
                    .expect("pager warmup lock poisoned");
            }
        }
    }

    fn run(self) {
        while let Some((views, revision)) = self.next_request() {
            for mode in [PagerLineNumberMode::Rendered, PagerLineNumberMode::Source] {
                if !self.is_current(revision) {
                    break;
                }
                // Cache failures for the normal on-demand error path without changing the display.
                let _ = views.view(mode);
            }
        }
    }
}

#[cfg(test)]
mod tests;
