use super::*;

#[test]
#[cfg(any(feature = "dynamic_output", feature = "static_output"))]
fn set_data() {
    let mut ps = PagerState::new().unwrap();
    let ev = Command::SetData(TEST_STR.to_string());
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        ev,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    assert_eq!(ps.screen.formatted_lines, vec![TEST_STR.to_string()]);
}

#[test]
fn append_str() {
    let mut ps = PagerState::new().unwrap();
    let ev1 = Command::AppendData(format!("{TEST_STR}\n"));
    let ev2 = Command::AppendData(TEST_STR.to_string());
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        ev1,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    handle_event(
        ev2,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(
        ps.screen.formatted_lines,
        vec![TEST_STR.to_string(), TEST_STR.to_string()]
    );
}

#[test]
#[cfg(feature = "static_output")]
fn set_run_no_overflow() {
    let mut ps = PagerState::new().unwrap();
    let ev = Command::SetRunNoOverflow(true);
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        ev,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert!(ps.run_no_overflow);
}

#[test]
fn add_exit_callback() {
    let mut ps = PagerState::new().unwrap();
    let ev = Command::AddExitCallback(Box::new(|| println!("Hello World")));
    let mut command_queue = CommandQueue::new_zero();

    handle_event(
        ev,
        &mut ps,
        &mut command_queue,
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(ps.exit_callbacks.len(), 1);
}
