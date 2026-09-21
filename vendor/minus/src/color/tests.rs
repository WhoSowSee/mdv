use super::{ColorDepth, palette::Color};

#[test]
fn palette_conversion_preserves_indices_and_reduces_rgb() {
    for index in 0..=255 {
        assert_eq!(Color::Indexed(index).index(ColorDepth::Ansi256), index);
        let reduced = Color::Indexed(index).index(ColorDepth::Ansi16);
        assert!(reduced < 16);
        if index < 16 {
            assert_eq!(reduced, index);
        }
    }
    for (rgb, ansi16, ansi256) in [
        ([255, 0, 0], 9, 196),
        ([0, 255, 0], 10, 46),
        ([0, 0, 255], 12, 21),
        ([128, 128, 128], 8, 244),
    ] {
        assert_eq!(Color::Rgb(rgb).index(ColorDepth::Ansi16), ansi16);
        assert_eq!(Color::Rgb(rgb).index(ColorDepth::Ansi256), ansi256);
    }
    for (rgb, ansi16) in [
        ([126, 156, 216], 12),
        ([122, 168, 159], 14),
        ([192, 163, 110], 3),
        ([152, 187, 108], 10),
        ([228, 104, 118], 9),
        ([149, 127, 184], 13),
        ([220, 215, 186], 7),
        ([26, 27, 38], 0),
    ] {
        assert_eq!(Color::Rgb(rgb).index(ColorDepth::Ansi16), ansi16, "{rgb:?}");
    }
}

#[test]
fn sgr_conversion_preserves_controls_and_handles_color_boundaries() {
    for (input, expected) in [
        (
            "\x1b[?25l\x1b[1;38;2;255;0;0;48;5;21m\x1bПривет 🌍\x1b[0m\x1b[?25h",
            "\x1b[?25l\x1b[1;91;104m\x1bПривет 🌍\x1b[0m\x1b[?25h",
        ),
        ("\x1b[38:2::255:0:0mred", "\x1b[91mred"),
        ("\x1b[38:5:9;4:3mred", "\x1b[91;4:3mred"),
        ("\x1b[4;58;2;255;0;0munderlined", "\x1b[4munderlined"),
        (
            "\x1b[38;2;25;25;25m\x1b[48;2;20;20;20mtext\x1b[49mnext",
            "\x1b[30m\x1b[40m\x1b[97mtext\x1b[49m\x1b[30mnext",
        ),
        (
            "\x1b[38;2;20;20;20;48;2;20;20;20mhidden",
            "\x1b[30;40mhidden",
        ),
    ] {
        assert_eq!(ColorDepth::Ansi16.adapt(input), expected);
    }
    let rgb = "\x1b[38;2;255;0;0mred";
    assert_eq!(ColorDepth::TrueColor.adapt(rgb), rgb);
    assert_eq!(ColorDepth::Ansi256.adapt(rgb), "\x1b[38;5;196mred");
    for input in [
        "\x1b]8;;https://example.com/\x1b[38;2;255;0;0m\x1b\\link\x1b]8;;\x1b\\",
        "\x1bPpayload\x1b[38;5;196m\x1b\\",
        "\x1b[38;2;999;0;0m",
        "\x1b[38;2;1",
    ] {
        assert_eq!(ColorDepth::Ansi16.adapt(input), input);
    }
}
