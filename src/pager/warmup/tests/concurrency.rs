use super::*;

#[test]
fn an_active_render_handles_switch_refresh_and_exit() {
    for action in ["switch", "refresh", "exit"] {
        let warmup = ViewWarmup::new().unwrap();
        let (sent, events) = mpsc::channel();
        let (finish, wait) = mpsc::channel();
        let wait = Mutex::new(wait);
        let views = PagerLineNumberViews::new(
            Off,
            Arc::new(move |mode| {
                sent.send(mode).unwrap();
                if mode == Rendered {
                    wait.lock().unwrap().recv_timeout(TIMEOUT).unwrap();
                }
                Ok(PagerDisplay::new(format!("{mode:?}\n"), vec![Some(1)]))
            }),
        );
        let mut current = document(views);
        current.attach_warmup(warmup.handle.clone());
        current.display_snapshot().unwrap();
        expect_modes(&events, &[Off]);
        let current = Arc::new(RwLock::new(current));
        warmup.handle.start();
        expire(&warmup.handle);
        expect_modes(&events, &[Rendered]);
        assert!(
            current.try_write().is_ok(),
            "{action}: background render holds document lock"
        );
        match action {
            "switch" => {
                let target = current.clone();
                let (locked, ready) = mpsc::channel();
                let switch = thread::spawn(move || {
                    let mut current = target.write().unwrap();
                    locked.send(()).unwrap();
                    current.cycle_line_number_mode().unwrap();
                    current.display_snapshot().unwrap().0
                });
                ready.recv_timeout(TIMEOUT).unwrap();
                finish.send(()).unwrap();
                assert_eq!(switch.join().unwrap(), "Rendered\n");
                expect_modes(&events, &[Source]);
                assert!(events.try_recv().is_err(), "switch duplicated a render");
            }
            "refresh" => {
                let (replacement, new_events) = observed_views(Off);
                crate::pager::operations::replace_document(&current, document(replacement))
                    .unwrap();
                expect_modes(&new_events, &[Off]);
                current.read().unwrap().display_snapshot().unwrap();
                expire(&warmup.handle);
                finish.send(()).unwrap();
                expect_modes(&new_events, &[Rendered, Source]);
                drop(current);
                assert!(
                    matches!(
                        events.recv_timeout(TIMEOUT),
                        Err(RecvTimeoutError::Disconnected)
                    ),
                    "old source mode rendered after refresh"
                );
            }
            "exit" => {
                let before = Instant::now();
                drop(warmup);
                assert!(before.elapsed() < Duration::from_secs(1));
                finish.send(()).unwrap();
                drop(current);
                assert!(
                    matches!(
                        events.recv_timeout(TIMEOUT),
                        Err(RecvTimeoutError::Disconnected)
                    ),
                    "source mode rendered after exit"
                );
            }
            _ => unreachable!(),
        }
    }
}
