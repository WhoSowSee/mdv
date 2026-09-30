use parking_lot::Mutex;
use std::{panic, sync::Arc};

type Hook = Box<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync + 'static>;

pub(super) fn with_cleanup<T>(
    cleanup: impl Fn() + Send + Sync + 'static,
    operation: impl FnOnce() -> T,
) -> T {
    let previous: Arc<Mutex<Option<Hook>>> = Arc::new(Mutex::new(Some(panic::take_hook())));
    let active = Arc::clone(&previous);
    panic::set_hook(Box::new(move |info| {
        cleanup();
        if let Some(previous) = active.lock().as_ref() {
            previous(info);
        }
    }));
    let result = panic::catch_unwind(panic::AssertUnwindSafe(operation));
    let previous = previous
        .lock()
        .take()
        .expect("session owns the previous panic hook");
    panic::set_hook(previous);
    match result {
        Ok(value) => value,
        Err(payload) => panic::resume_unwind(payload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn repeated_sessions_restore_the_original_hook_after_success_and_panic() {
        const CHILD_ENV: &str = "MDV_MINUS_PANIC_HOOK_REGRESSION";
        if std::env::var_os(CHILD_ENV).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "repeated_sessions_restore_the_original_hook_after_success_and_panic",
                    "--nocapture",
                ])
                .env(CHILD_ENV, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let original = panic::take_hook();
        let reports = Arc::new(AtomicUsize::new(0));
        let counter = reports.clone();
        panic::set_hook(Box::new(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
        let cleanups = Arc::new(AtomicUsize::new(0));
        for should_panic in [false, true, false, true] {
            let counter = cleanups.clone();
            let result = panic::catch_unwind(|| {
                with_cleanup(
                    move || {
                        counter.fetch_add(1, Ordering::SeqCst);
                    },
                    || {
                        if should_panic {
                            panic!("session panic");
                        }
                    },
                )
            });
            assert_eq!(result.is_err(), should_panic);
        }
        let _ = panic::catch_unwind(|| panic!("outside session"));
        let reports = reports.load(Ordering::SeqCst);
        let cleanups = cleanups.load(Ordering::SeqCst);
        panic::set_hook(original);
        assert_eq!(reports, 3);
        assert_eq!(cleanups, 2);
    }
}
