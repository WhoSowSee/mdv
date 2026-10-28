use super::SgrState;

#[test]
fn sgr_prefix_compacts_attributes() {
    for (input, expected) in [
        ("\x1b[1;2;22;1m", "\x1b[22m\x1b[1m"),
        ("\x1b[1;31m\x1b[0;3;32m", ""),
        ("\x1b[1;31m\x1b[m", ""),
        ("\x1b[1;;4m", ""),
        ("\x1b[3;38;2;0;255;0m", "\x1b[3m\x1b[38;2;0;255;0m"),
        ("\x1b[48;2;0;0;0;58;5;0m", "\x1b[48;2;0;0;0m\x1b[58;5;0m"),
        (
            "\x1b[4;38;5;42;48;2;1;2;3m",
            "\x1b[4m\x1b[38;5;42m\x1b[48;2;1;2;3m",
        ),
        (
            "\x1b[3;4;31;44m\x1b[23;24;39;49m",
            "\x1b[23m\x1b[24m\x1b[39m\x1b[49m",
        ),
        (
            "\x1b[5;7;8;9m\x1b[25;27;28;29m",
            "\x1b[25m\x1b[27m\x1b[28m\x1b[29m",
        ),
        ("\x1b[91;104;58;5;42m", "\x1b[91m\x1b[104m\x1b[58;5;42m"),
        (
            "\x1b[38:2::1:255:1;48:5:42m",
            "\x1b[38;2;1;255;1m\x1b[48;5;42m",
        ),
        ("\x1b[4:3;58:2:1:10:20m", "\x1b[4:3m\x1b[58;2;1;10;20m"),
    ] {
        let mut state = SgrState::default();
        state.update(input);
        assert_eq!(state.prefix(), expected, "{input:?}");
    }
}

#[test]
fn sgr_prefix_replaces_previous_attributes_without_retaining_history() {
    let mut state = SgrState::default();
    for _ in 0..1000 {
        state.update("\x1b[1;31;44mred\x1b[22;3;38;2;1;255;1;49mgreen");
    }
    assert_eq!(state.prefix(), "\x1b[22m\x1b[3m\x1b[38;2;1;255;1m\x1b[49m");
}

#[test]
fn sgr_tracking_does_not_interpret_other_controls_or_incomplete_colors() {
    for suffix in [
        "\x1b]8;;https://example.com/0\x1b\\",
        "\x1b[2K",
        "\x1b[38;2;1;255m",
        "\x1b[38;5;256m",
    ] {
        let mut state = SgrState::default();
        state.update("\x1b[1;31m");
        state.update(suffix);
        assert_eq!(state.prefix(), "\x1b[1m\x1b[31m", "{suffix:?}");
    }
}
