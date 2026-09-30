use super::ast::MathDiagnostic;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(crate) struct MathDiagnostics {
    reported: Arc<Mutex<HashSet<u64>>>,
}

impl MathDiagnostics {
    pub(crate) fn report(&self, source: &str, diagnostics: &[MathDiagnostic]) {
        let Some(first) = diagnostics.first() else {
            return;
        };
        self.report_once((source, first.offset, &first.message), || {
            let remainder = diagnostics.len() - 1;
            if remainder == 0 {
                log::warn!("TeX at byte {}: {}", first.offset, first.message);
            } else {
                log::warn!(
                    "TeX at byte {}: {} (and {} more issue{})",
                    first.offset,
                    first.message,
                    remainder,
                    if remainder == 1 { "" } else { "s" }
                );
            }
        });
    }

    pub(crate) fn report_overflow(&self, source: &str, width: usize, available: usize) {
        self.report_once(("overflow", source), || {
            log::warn!(
                "Math layout is {} columns wide but only {} columns are available",
                width,
                available
            );
        });
    }

    fn report_once(&self, key: impl Hash, report: impl FnOnce()) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let first_report = self
            .reported
            .lock()
            .expect("math diagnostic lock poisoned")
            .insert(hasher.finish());
        if first_report {
            report();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn warnings_are_shared_between_threads_and_reset_for_each_document() {
        let reports = Arc::new(AtomicUsize::new(0));
        for _ in 0..2 {
            let diagnostics = MathDiagnostics::default();
            let threads = (0..3)
                .map(|_| {
                    let diagnostics = diagnostics.clone();
                    let reports = reports.clone();
                    std::thread::spawn(move || {
                        for formula in 0..8 {
                            diagnostics.report_once(formula, || {
                                reports.fetch_add(1, Ordering::SeqCst);
                            });
                        }
                    })
                })
                .collect::<Vec<_>>();
            for thread in threads {
                thread.join().unwrap();
            }
        }
        assert_eq!(reports.load(Ordering::SeqCst), 16);
    }
}
