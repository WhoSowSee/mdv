use super::*;
use crate::{
    cli::OutputStyle,
    pager::{PagerContent, PagerDisplay},
};
use PagerLineNumberMode::{Off, Rendered, Source};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};

const TIMEOUT: Duration = Duration::from_secs(3);

fn observed_views(
    mode: PagerLineNumberMode,
) -> (PagerLineNumberViews, Receiver<PagerLineNumberMode>) {
    let (sent, events) = mpsc::channel();
    let views = PagerLineNumberViews::new(
        mode,
        Arc::new(move |mode| {
            sent.send(mode).unwrap();
            Ok(PagerDisplay::new(format!("{mode:?}\n"), vec![Some(1)]))
        }),
    );
    (views, events)
}

fn expect_modes(events: &Receiver<PagerLineNumberMode>, modes: &[PagerLineNumberMode]) {
    for mode in modes {
        assert_eq!(events.recv_timeout(TIMEOUT).unwrap(), *mode);
    }
}

fn job(handle: &WarmupHandle) -> (PagerLineNumberViews, Instant, u64) {
    let state = handle.shared.state.lock().unwrap();
    let request = state.request.as_ref().unwrap();
    (request.views.clone(), request.deadline, state.revision)
}

fn expire(handle: &WarmupHandle) {
    let mut state = handle.shared.state.lock().unwrap();
    state.request.as_mut().unwrap().deadline = Instant::now();
    handle.shared.changed.notify_one();
}

fn document(views: PagerLineNumberViews) -> PagerDocument {
    PagerDocument::from_content(PagerContent::LineNumbers(views), OutputStyle::Disabled)
}

#[test]
fn warmup_starts_after_first_screen_and_cancels_on_exit() {
    for initial in [Off, Rendered, Source] {
        let warmup = ViewWarmup::new().unwrap();
        let (views, events) = observed_views(initial);
        let mut current = document(views);
        current.attach_warmup(warmup.handle.clone());
        let output = current.display_snapshot().unwrap().0;
        expect_modes(&events, &[initial]);
        expire(&warmup.handle);
        assert!(matches!(
            events.recv_timeout(Duration::from_millis(30)),
            Err(RecvTimeoutError::Timeout)
        ));
        let before = Instant::now();
        warmup.handle.start();
        let deadline = job(&warmup.handle).1;
        assert!(deadline >= before + Duration::from_secs(5));
        assert!(deadline <= Instant::now() + Duration::from_secs(5));
        current.display_snapshot().unwrap();
        assert_eq!(job(&warmup.handle).1, deadline);
        assert!(events.try_recv().is_err());
        expire(&warmup.handle);
        let missing: Vec<_> = [Rendered, Source]
            .into_iter()
            .filter(|mode| *mode != initial)
            .collect();
        expect_modes(&events, &missing);
        assert_eq!(current.display_snapshot().unwrap().0, output);
        assert_eq!(current.line_number_mode(), Some(initial));
        for _ in 0..6 {
            current.cycle_line_number_mode().unwrap();
        }
        let expected: Vec<_> = [Off].into_iter().filter(|mode| *mode != initial).collect();
        expect_modes(&events, &expected);
        assert!(events.try_recv().is_err());
    }
    let warmup = ViewWarmup::new().unwrap();
    let handle = warmup.handle.clone();
    let (views, events) = observed_views(Off);
    handle.schedule(views);
    handle.start();
    let before = Instant::now();
    drop(warmup);
    assert!(before.elapsed() < Duration::from_secs(1));
    assert!(matches!(
        events.recv_timeout(TIMEOUT),
        Err(RecvTimeoutError::Disconnected)
    ));
    assert!(handle.next_request().is_none());
}

#[test]
fn refresh_restarts_delay_for_the_replacement_document() {
    let warmup = ViewWarmup::new().unwrap();
    let (views, old_events) = observed_views(Off);
    let mut current = document(views);
    current.attach_warmup(warmup.handle.clone());
    current.display_snapshot().unwrap();
    expect_modes(&old_events, &[Off]);
    warmup.handle.start();
    let original = job(&warmup.handle).1;
    let current = RwLock::new(current);
    let (replacement, new_events) = observed_views(Off);
    crate::pager::operations::replace_document(&current, document(replacement)).unwrap();
    expect_modes(&new_events, &[Off]);
    assert!(warmup.handle.shared.state.lock().unwrap().request.is_none());
    let before = Instant::now();
    current.read().unwrap().display_snapshot().unwrap();
    assert!(job(&warmup.handle).1 >= before + WARMUP_DELAY);
    assert!(job(&warmup.handle).1 >= original);
    expire(&warmup.handle);
    expect_modes(&new_events, &[Rendered, Source]);
    assert!(old_events.try_recv().is_err());
}

#[test]
fn background_errors_are_cached_without_changing_the_display() {
    let warmup = ViewWarmup::new().unwrap();
    let (sent, events) = mpsc::channel();
    let views = PagerLineNumberViews::new(
        Off,
        Arc::new(move |mode| {
            sent.send(mode).unwrap();
            if mode == Rendered {
                anyhow::bail!("background render failed");
            }
            Ok(PagerDisplay::new("current\n".into(), vec![Some(1)]))
        }),
    );
    let mut current = document(views);
    current.attach_warmup(warmup.handle.clone());
    current.display_snapshot().unwrap();
    expect_modes(&events, &[Off]);
    warmup.handle.start();
    expire(&warmup.handle);
    expect_modes(&events, &[Rendered, Source]);
    assert_eq!(current.display_snapshot().unwrap().0, "current\n");
    for _ in 0..2 {
        assert!(
            current
                .cycle_line_number_mode()
                .unwrap_err()
                .to_string()
                .contains("background render failed")
        );
        assert_eq!(current.line_number_mode(), Some(Off));
    }
    assert!(events.try_recv().is_err());
}

#[test]
fn width_changes_reset_the_job_but_numbering_changes_do_not() {
    let warmup = ViewWarmup::new().unwrap();
    let mut current = crate::pager::tests::markdown_document(100);
    current.attach_warmup(warmup.handle.clone());
    current.layout_snapshot(100).unwrap();
    warmup.handle.start();
    let (original, deadline, revision) = job(&warmup.handle);
    current.cycle_line_number_mode().unwrap();
    current.layout_snapshot(100).unwrap();
    assert!(warmup.handle.is_current(revision));
    assert_eq!(job(&warmup.handle).1, deadline);
    let before = Instant::now();
    let output = current.layout_snapshot(64).unwrap().0;
    let (narrow, deadline, _) = job(&warmup.handle);
    assert!(!narrow.shares_cache_with(&original));
    assert!(deadline >= before + WARMUP_DELAY);
    assert!(!warmup.handle.is_current(revision));
    assert_eq!(current.line_number_mode(), Some(Rendered));
    assert_eq!(current.display_snapshot().unwrap().0, output);
    current.layout_snapshot(100).unwrap();
    assert!(job(&warmup.handle).0.shares_cache_with(&original));
}

mod concurrency;
