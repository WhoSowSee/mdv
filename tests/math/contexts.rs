use super::*;

#[test]
fn extended_math_trailing_space_does_not_leak_internal_placeholders() {
    let output = render(r"\( x^2|y \)", &[]);
    assert_eq!(output, "x²|y\n");
}

#[test]
fn source_numbered_fenced_math_has_the_same_math_geometry() {
    for (source, expected) in [
        ("\x60\x60\x60math\nx^\n2\n\x60\x60\x60", "x²"),
        ("\x60\x60\x60math\n\\frac\n{a}\n{b}\n\x60\x60\x60", "a"),
    ] {
        let plain = render(source, &["--math-block-style", "pretty"]);
        let numbered = render(
            source,
            &[
                "--math-block-style",
                "pretty",
                "--line-numbers",
                "source;separator",
            ],
        );
        assert!(plain.contains(expected), "{plain}");
        let mapped = numbered
            .lines()
            .map(|line| line.split_once(" │ ").unwrap())
            .collect::<Vec<_>>();
        let geometry = mapped
            .iter()
            .map(|(_, line)| *line)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(geometry, plain.trim_end(), "{numbered}");
        let numbered_lines = mapped
            .iter()
            .filter(|(number, _)| !number.trim().is_empty())
            .collect::<Vec<_>>();
        assert_eq!(numbered_lines.len(), 1, "{numbered}");
        assert_eq!(numbered_lines[0].0.trim(), "2", "{numbered}");
        assert!(numbered_lines[0].1.contains(expected), "{numbered}");
    }
}

#[test]
fn reference_destination_does_not_hide_following_math() {
    let output = render(
        "[label]: https://example.com \\(x^2\\) prose\n\n[label]\n",
        &[],
    );
    assert!(output.contains("x²"), "{output}");
}

#[test]
fn math_is_kept_in_callout_label_order() {
    let linked = "[value $x^2$](https://example.com)";
    let mut cases = Vec::new();
    cases.push(("value $x^2$", "value x²", "simple", "clickable"));
    for style in ["simple", "pretty", "pretty:label-inside"] {
        cases.push((linked, "value x²", style, "clickable"));
    }
    for link_style in ["inline", "inlinetable", "hide", "endtable"] {
        cases.push((linked, "value x²", "simple", link_style));
    }
    cases.push((
        "before [value $x^2$](https://example.com) after",
        "before value x² after",
        "pretty:label-inside",
        "inline",
    ));
    for (label, expected, style, link_style) in cases {
        let output = render(
            &format!("> [!NOTE] {label}\n> body\n"),
            &["--callout-style", style, "--link-style", link_style],
        );
        assert_eq!(
            output.matches(expected).count(),
            1,
            "{style}, {link_style}:\n{output}"
        );
        assert!(output.contains("body"), "{output}");
        assert_eq!(output.matches("value").count(), 1, "{output}");
        assert_eq!(output.matches("x²").count(), 1, "{output}");
    }
}

#[test]
fn html_sup_sub_keep_semantic_spaces() {
    let output = render("A<sup>1 2</sup>B<sub>3 4</sub>C\n", &["--render-html"]);
    assert!(output.contains("A¹ ²B₃ ₄C"), "{output}");
}

#[test]
fn nested_pretty_callouts_keep_math_borders_intact() {
    let depth = 3;
    let mut source = String::new();
    for level in 1..=depth {
        source.push_str(&format!("{}[!NOTE]\n", "> ".repeat(level)));
    }
    let prefix = "> ".repeat(depth);
    for line in [r"\[", r"\frac{abcdefghijklmnop}{qrstuvwxyz}", r"\]"] {
        source.push_str(&format!("{prefix}{line}\n"));
    }
    let output = render(&source, &["--cols", "16", "--math-block-style", "pretty"]);
    assert_framed_fraction(&output);
}

#[test]
fn inline_atoms_finish_with_impossible_widths() {
    for (source, cols, mode, expected) in [
        ("> $xxxx$", "2", "word", "xx"),
        ("$界$", "1", "char", "界"),
        ("\x60👨‍👩‍👧‍👦界\x60", "1", "char", "👨‍👩‍👧‍👦"),
    ] {
        let output = mdv_cmd()
            .args(["--no-colors", "--cols", cols, "--wrap", mode])
            .write_stdin(source)
            .timeout(std::time::Duration::from_secs(5))
            .output()
            .unwrap();
        assert!(output.status.success());
        let output = strip_ansi(&String::from_utf8(output.stdout).unwrap());
        assert!(output.contains(expected), "{output}");
        if source.contains("xxxx") {
            assert_eq!(output.matches('x').count(), 4, "{output}");
        }
    }
}

#[test]
fn reverse_moves_display_math_as_one_block() {
    let output = render(
        "Before\n\n\\[\n\\frac{a}{b}\n\\]\n\nAfter\n",
        &["--reverse", "--math-block-style", "simple"],
    );
    let after = output.find("After").unwrap();
    let before = output.find("Before").unwrap();
    assert!(after < before, "{output}");
    assert!(
        output[after..before].contains("│  a\n│  ─\n│  b"),
        "{output}"
    );
}
