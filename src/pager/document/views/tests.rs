use super::*;
use crate::{
    cli::OutputStyle,
    pager::{PagerContent, PagerDocument},
};
use PagerLineNumberMode::{Off, Rendered, Source};
use std::sync::{
    RwLock,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

type RenderCounts = Arc<[AtomicUsize; 3]>;

fn counted_document(
    mode: PagerLineNumberMode,
    label: &'static str,
) -> (PagerDocument, RenderCounts) {
    let counts: RenderCounts = Arc::default();
    let renders = counts.clone();
    let views = PagerLineNumberViews::new(
        mode,
        Arc::new(move |mode| {
            renders[mode as usize].fetch_add(1, Ordering::SeqCst);
            Ok(PagerDisplay::new(
                format!("{label} {mode:?}\n"),
                vec![Some(1)],
            ))
        }),
    );
    (
        PagerDocument::from_content(PagerContent::LineNumbers(views), OutputStyle::Disabled),
        counts,
    )
}

fn counts(values: &RenderCounts) -> [usize; 3] {
    std::array::from_fn(|index| values[index].load(Ordering::SeqCst))
}

fn cache(document: &PagerDocument) -> &[CachedDisplay; 3] {
    match &document.content {
        PagerContent::LineNumbers(views) => &views.displays,
        _ => panic!("missing pager views"),
    }
}

fn ready(document: &PagerDocument) -> [bool; 3] {
    cache(document).each_ref().map(|view| view.get().is_some())
}

#[test]
fn snapshots_navigation_cycles_and_refresh_share_the_expected_cache() {
    for (index, mode) in [Off, Rendered, Source].into_iter().enumerate() {
        let (mut current, rendered) = counted_document(mode, "old");
        assert_eq!(counts(&rendered), [0; 3]);
        let mut expected = [0; 3];
        expected[index] = 1;
        for _ in 0..3 {
            let (output, mut navigation) = current.display_snapshot().unwrap();
            assert_eq!(output, format!("old {mode:?}\n"));
            assert_eq!(counts(&rendered), expected);
            navigation.as_mut().unwrap().prepare_source_view().unwrap();
            expected[2] = 1;
            assert_eq!(counts(&rendered), expected);
        }
        for _ in 0..6 {
            assert!(current.cycle_line_number_mode().unwrap());
            current.display_snapshot().unwrap();
        }
        assert_eq!(counts(&rendered), [1; 3]);
        while current.line_number_mode() != Some(Source) {
            current.cycle_line_number_mode().unwrap();
        }
        let current = RwLock::new(current);
        let (replacement, refreshed) = counted_document(Off, "new");
        crate::pager::operations::replace_document(&current, replacement).unwrap();
        assert_eq!(counts(&refreshed), [0, 0, 1]);
        assert_eq!(
            current.read().unwrap().display_snapshot().unwrap().0,
            "new Source\n"
        );
        assert_eq!(counts(&refreshed), [0, 0, 1]);
    }
}

#[test]
fn refresh_handles_failure_and_concurrent_numbering_change() {
    let timeout = std::time::Duration::from_secs(3);
    for change_mode in [false, true] {
        let (current, _) = counted_document(Source, "old");
        let current = Arc::new(RwLock::new(current));
        let (started, ready) = mpsc::channel();
        let (resume, wait) = mpsc::channel();
        let wait = std::sync::Mutex::new(wait);
        let views = PagerLineNumberViews::new(
            Off,
            Arc::new(move |mode| {
                if mode == Source {
                    started.send(mode).unwrap();
                    if !change_mode {
                        anyhow::bail!("refreshed source render failed");
                    }
                    wait.lock().unwrap().recv_timeout(timeout).unwrap();
                }
                Ok(PagerDisplay::new(format!("new {mode:?}\n"), vec![Some(1)]))
            }),
        );
        let target = current.clone();
        let worker = std::thread::spawn(move || {
            crate::pager::operations::replace_document(
                &target,
                PagerDocument::from_content(
                    PagerContent::LineNumbers(views),
                    OutputStyle::Disabled,
                ),
            )
        });
        assert_eq!(ready.recv_timeout(timeout).unwrap(), Source);
        if change_mode {
            let mut current = current
                .try_write()
                .expect("refresh holds document lock while rendering");
            current.cycle_line_number_mode().unwrap();
            assert_eq!(current.line_number_mode(), Some(Off));
            resume.send(()).unwrap();
        }
        let result = worker.join().unwrap();
        let current = current.read().unwrap();
        if change_mode {
            result.unwrap();
            assert_eq!(current.line_number_mode(), Some(Off));
            assert_eq!(current.display_snapshot().unwrap().0, "new Off\n");
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("refreshed source render failed")
            );
            assert_eq!(current.line_number_mode(), Some(Source));
            assert_eq!(current.display_snapshot().unwrap().0, "old Source\n");
        }
    }
}

#[test]
fn width_changes_render_only_selected_modes_and_reuse_cached_widths() {
    let mut current = crate::pager::tests::markdown_document(100);
    let full = current.layout_snapshot(100).unwrap().0;
    assert_eq!(ready(&current), [true, false, false]);
    let saved = cache(&current)[0].clone();
    current.layout_snapshot(64).unwrap();
    assert_eq!(ready(&current), [true, false, false]);
    assert_eq!(current.layout_snapshot(100).unwrap().0, full);
    assert!(Arc::ptr_eq(&saved, &cache(&current)[0]));
    current.cycle_line_number_mode().unwrap();
    current.layout_snapshot(64).unwrap();
    assert_eq!(ready(&current), [true, true, false]);
}
