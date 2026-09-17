use super::ast::MathDiagnostic;
use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub(crate) struct MathDiagnostics {
    reported: RefCell<HashSet<u64>>,
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
        if self.reported.borrow_mut().insert(hasher.finish()) {
            report();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn warnings_are_deduplicated_for_all_layouts_and_reset_for_each_document() {
        let reports = Cell::new(0);
        for _ in 0..2 {
            let diagnostics = MathDiagnostics::default();
            for _ in 0..3 {
                for formula in 0..300 {
                    diagnostics.report_once(formula, || reports.set(reports.get() + 1));
                }
            }
        }
        assert_eq!(reports.get(), 600);
    }
}
