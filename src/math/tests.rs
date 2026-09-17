use super::*;

#[test]
fn inline_renderer_covers_extended_commands() {
    let rendered = render_math(
        r"\dfrac{a+b}{c+d}, \hat{x}, \bar{y}, \min_x f(x), \mathbb{P}, \underbrace{x}_n^2",
        MathMode::Inline,
    );

    for expected in ["(a+b)⁄(c+d)", "x̂", "y̅", "minₓ", "ℙ", "underbrace(x, n)²"] {
        assert!(
            rendered.contains(expected),
            "missing {expected:?}: {rendered}"
        );
    }
}

#[test]
fn inline_renderer_preserves_delimiters_scripts_spacing_and_fonts() {
    assert_eq!(
        render_math(
            r"\mathrm{KL}(P\|Q), \lVert x\rVert_2, \langle u,v\rangle",
            MathMode::Inline,
        ),
        "KL(P‖Q), ‖x‖₂, ⟨u,v⟩"
    );
    assert_eq!(
        render_math(
            r"\frac{d}{dx}\sin x, \log\frac{p(x)}{q(x)}",
            MathMode::Inline,
        ),
        "d⁄dx sin x, log (p(x)⁄q(x))"
    );
    assert_eq!(
        render_math(
            r"\lim_{x\to 0}, x_{i,j}, t_{obs}, \int_0^{\pi}",
            MathMode::Inline,
        ),
        "lim_(x→0), xᵢ,ⱼ, tₒᵦₛ, ∫₀^π"
    );
    assert_eq!(
        render_math(
            r"\mathcal{F}\mathcal{L}\mathscr{N}, \mathbf{w}, \boldsymbol{\mu}, \mathfrak{Rz}",
            MathMode::Inline,
        ),
        "ℱℒ𝒩, 𝐰, 𝛍, ℜ𝔷"
    );
}

#[test]
fn display_renderer_preserves_root_operator_and_atomic_layout() {
    assert_eq!(
        render_math(r"\overline{z}=a-bi", MathMode::Display),
        "z̅=a-bi"
    );
    assert_eq!(
        render_math(r"\sqrt{2\pi\sigma^2}", MathMode::Inline),
        "√(2πσ²)"
    );
    assert_eq!(
        render_math(r"\sqrt{2\pi\sigma^2}", MathMode::Display),
        "√(2πσ²)"
    );
    assert_eq!(
        render_math(
            r"\mathrm{arg\,max}_{x\in\mathcal X} f(x)",
            MathMode::Display,
        ),
        "arg max_(x∈𝒳) f(x)"
    );
    assert_eq!(
        render_math(
            r"\operatorname*{arg\,max}_{x\in\mathcal X} f(x)",
            MathMode::Display,
        ),
        "arg max f(x)\n  x∈𝒳"
    );
    assert_eq!(
        render_math(r"\frac{a_C^c a_D^d}{a_A^a a_B^b}", MathMode::Display,),
        "a_Cᶜ a_Dᵈ\n─────────\na_Aᵃ a_Bᵇ"
    );
    assert_eq!(
        render_math(r"\binom{n}{k}", MathMode::Display),
        "⎛n⎞\n⎜ ⎟\n⎝k⎠"
    );
}

#[test]
fn unknown_and_invalid_tex_preserve_source() {
    assert_eq!(
        render_math(r"\unknown{a_b}", MathMode::Inline),
        r"\unknown{a_b}"
    );
    assert_eq!(
        render_math(r"\unknown*{a_b}", MathMode::Inline),
        r"\unknown*{a_b}"
    );
    assert_eq!(render_math(r"\frac{1}{", MathMode::Display), r"\frac{1}{");
    assert_eq!(render_math("^2", MathMode::Inline), "^2");
    assert_eq!(render_math("a~b", MathMode::Inline), "a b");
}

#[test]
fn scripts_preserve_group_boundaries_and_ignored_spacing() {
    for mode in [MathMode::Inline, MathMode::Display] {
        for (formula, expected) in [
            (r"e^{\mathcal X}", "e^𝒳"),
            (r"e^{i\mathcal X}", "e^(i𝒳)"),
            (r"e^{,}", "e^,"),
            (r"x_{i\in X}", "x_(i∈X)"),
            (r"x_{i\to 0}", "x_(i→0)"),
            (r"x^{\frac{a}{b}}", "x^(a⁄b)"),
        ] {
            assert_eq!(render_math(formula, mode), expected, "{formula}, {mode:?}");
        }
        assert_ne!(
            render_math(r"e^{\mathcal X}", mode),
            render_math(r"e\mathcal X", mode)
        );
    }
    assert_eq!(render_math(r"e^{i\pi}", MathMode::Inline), "e^(iπ)");
    assert_eq!(render_math(r"e^i\pi", MathMode::Inline), "eⁱπ");
    assert_eq!(render_math(r"x _1", MathMode::Inline), "x₁");
    assert_eq!(render_math("x% ignored\n_1", MathMode::Inline), "x₁");
}

#[test]
fn unsupported_commands_remain_literal_inside_styles_and_spaced_groups() {
    for mode in [MathMode::Inline, MathMode::Display] {
        for (formula, expected) in [
            (r"\mathbf{\unknown*{abc}}", r"\unknown*{abc}"),
            ("\\mathbf{\\unknown{a \nb}}", "\\unknown{a \nb}"),
            ("\\mathbf{\\unknown{a\n }}", "\\unknown{a\n }"),
            (r"\mathbf{x_1+\unknown{abc}+y}", r"𝐱₁+\unknown{abc}+𝐲"),
            (r"\mathbf{\mathbb{\unknown{abc}}}", r"\unknown{abc}"),
            (r"\mathbf{\operatorname{\unknown{abc}}}", r"\unknown{abc}"),
            (
                r"\mathbf{\operatorname*{a+\unknown{abc}+b}}",
                r"𝐚+\unknown{abc}+𝐛",
            ),
        ] {
            assert_eq!(render_math(formula, mode), expected, "{formula}, {mode:?}");
        }
        assert_eq!(render_math(r"\mathbf{\mathcal{A}\mu}", mode), "𝒜𝛍");
    }
    assert_eq!(
        render_math(r"\unknown {\frac{a}{b}}", MathMode::Inline),
        r"\unknown {\frac{a}{b}}"
    );
}

#[test]
fn delimiters_require_right_argument_and_support_standalone_commands() {
    assert_eq!(
        render_math(r"\left(x\right", MathMode::Inline),
        r"\left(x\right"
    );
    assert_eq!(render_math(r"\left. x\right)", MathMode::Inline), "x)");
    assert_eq!(render_math(r"\lfloor x\rfloor", MathMode::Inline), "⌊x⌋");
    assert_eq!(render_math(r"\lceil x\rceil", MathMode::Inline), "⌈x⌉");
}

#[test]
fn comments_before_arguments_are_not_rendered() {
    assert_eq!(render_math("\\sqrt% note\n{x}", MathMode::Inline), "√x");
    assert_eq!(render_math("\\frac% note\n{a}{b}", MathMode::Inline), "a⁄b");
}

#[test]
fn nested_inline_fractions_keep_their_grouping() {
    for (formula, expected) in [
        (r"\frac{1}{\frac{2}{3}}", "1⁄(2⁄3)"),
        (r"\frac{\frac{1}{2}}{3}", "(1⁄2)⁄3"),
        (r"\frac{(\frac{a}{b})+(\frac{c}{d})}{e}", "((a⁄b)+(c⁄d))⁄e"),
        (r"\frac{(\frac{a}{b}+\frac{c}{d})}{e}", "(a⁄b+c⁄d)⁄e"),
    ] {
        assert_eq!(
            render_math(formula, MathMode::Inline),
            expected,
            "{formula}"
        );
    }
    assert_eq!(
        render_math(
            r"\tfrac{(\tfrac{a}{b})+(\tfrac{c}{d})}{e}",
            MathMode::Display,
        ),
        "((a⁄b)+(c⁄d))⁄e"
    );
}

#[test]
fn operator_spacing_renders_nested_subtrees_a_bounded_number_of_times() {
    for depth in [18, 24] {
        let input = format!("{}x{}", r"\sum{".repeat(depth), "}".repeat(depth));
        super::rendering::reset_render_visits();
        render_math(&input, MathMode::Inline);
        assert!(super::rendering::render_visits() <= input.len());
    }
}

#[test]
fn physical_and_explicit_math_line_breaks_are_distinct() {
    assert_eq!(render_math("a\nb", MathMode::Display), "a b");
    assert_eq!(render_math(r"a\\b", MathMode::Display), "a\nb");
    assert_eq!(render_math(r"\text{a  b}", MathMode::Inline), "a  b");
    assert_eq!(render_math("\\text{a\nb}", MathMode::Inline), "a b");
    assert_eq!(render_math(r"\text{a\% b}", MathMode::Inline), "a% b");
    assert_eq!(render_math("\\text{a% ignored\nb}", MathMode::Inline), "ab");
    assert_eq!(render_math("a% ignored\nb", MathMode::Inline), "ab");
}

#[test]
fn nested_environment_delimiters_do_not_split_outer_cells() {
    let rendered = render_math(
        r"\begin{bmatrix}\frac{a}{b}&\begin{matrix}1&2\\3&4\end{matrix}\end{bmatrix}",
        MathMode::Display,
    );

    assert!(
        rendered.contains('⎡') && rendered.contains('⎣'),
        "{rendered}"
    );
    assert!(
        rendered.contains("1 2") && rendered.contains("3 4"),
        "{rendered}"
    );

    let escaped_marker = render_math(
        r"\begin{gather}a\\end{gather}\\b\end{gather}",
        MathMode::Display,
    );
    assert!(escaped_marker.ends_with('b'), "{escaped_marker}");

    assert_eq!(
        render_math(r"\begin{bmatrix*}1&2\end{bmatrix*}", MathMode::Inline),
        "[1, 2]"
    );
}

#[test]
fn comments_do_not_split_environment_cells_or_rows() {
    let rendered = render_math(
        "\\begin{matrix}a % & ignored\n& b\\end{matrix}",
        MathMode::Inline,
    );
    assert!(
        rendered.contains("a") && rendered.contains("b"),
        "{rendered}"
    );
    assert!(!rendered.contains("ignored"), "{rendered}");
}

#[test]
fn substack_diagnostics_use_absolute_utf8_offsets() {
    for input in [
        r"\substack{é \unknown}",
        r"é\begin{matrix}\substack{é \unknown}\end{matrix}",
    ] {
        let parsed = super::parser::parse_math(input);
        let diagnostic = parsed
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("unsupported command"))
            .expect("unsupported command diagnostic");
        assert_eq!(diagnostic.offset, input.find(r"\unknown").unwrap());
    }
}

#[test]
fn array_column_specification_controls_alignment_and_rules() {
    let rendered = render_math(
        r"\begin{array}{r|c}1&x\\20&yy\end{array}",
        MathMode::Display,
    );

    assert_eq!(rendered, " 1 │ x\n20 │ yy");
    assert_eq!(
        render_math(
            r"\begin{array}{|r|c|}1&x\\20&yy\end{array}",
            MathMode::Display
        ),
        "│  1 │ x  │\n│ 20 │ yy │"
    );
    assert_eq!(
        render_math(r"\begin{align}x&=1\\long&=2\end{align}", MathMode::Display),
        "   x =1\nlong =2"
    );
}

#[test]
fn flat_math_wraps_without_splitting_combining_graphemes() {
    let accented = "x\u{0302}x\u{0302}";
    assert_eq!(
        crate::math::wrap_flat_math(accented, 1, crate::utils::WrapMode::Character),
        "x\u{0302}\nx\u{0302}"
    );
    assert_eq!(
        crate::math::wrap_flat_math("a + b + c", 5, crate::utils::WrapMode::Word),
        "a + b\n+ c"
    );
    assert_eq!(
        crate::math::wrap_flat_math("unbroken", 4, crate::utils::WrapMode::Word),
        "unbroken"
    );
    assert_eq!(
        crate::math::wrap_flat_math("界", 1, crate::utils::WrapMode::Character),
        "界"
    );
}

#[test]
fn accents_use_a_shared_grapheme_and_width_contract() {
    assert_eq!(render_math(r"\hat{x}", MathMode::Inline), "x̂");
    assert_eq!(render_math(r"\hat{x}", MathMode::Display), "x̂");
    assert_eq!(render_math(r"\hat{界}", MathMode::Inline), "hat(界)");
    assert_eq!(render_math(r"\hat{界}", MathMode::Display), "^\n界");
    assert_eq!(
        render_math(r"\hat{\frac{a}{b}}", MathMode::Display),
        "^\na\n─\nb"
    );
}

#[test]
fn limits_modifier_stays_attached_to_its_operator() {
    assert_eq!(
        render_math(r"\sum\limits_{i=1}^{n}x_i", MathMode::Inline),
        "∑ᵢ₌₁ⁿ xᵢ"
    );
}

#[test]
fn excessive_nesting_preserves_input_without_recursing_forever() {
    let source = format!("{}x{}", "{".repeat(130), "}".repeat(130));

    assert_eq!(render_math(&source, MathMode::Display), source);

    let source = format!("{}x", "\\sqrt".repeat(256));
    assert_eq!(render_math(&source, MathMode::Display), source);
}
