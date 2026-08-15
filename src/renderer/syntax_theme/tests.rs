use super::*;

#[test]
fn reset_encodes_as_transparent_sentinel() {
    assert_eq!(
        transparent_for_reset(&Color::Reset),
        Some(SyntectColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0
        })
    );
}
