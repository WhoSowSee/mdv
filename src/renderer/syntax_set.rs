use once_cell::sync::Lazy;
use syntect::dumps::from_uncompressed_data;
use syntect::parsing::SyntaxSet;

const EMBEDDED_SYNTAX_SET: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/syntaxes.bin"));

/// Global cache of syntaxes to avoid unpacking them every time a renderer is created.
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    from_uncompressed_data::<SyntaxSet>(EMBEDDED_SYNTAX_SET).unwrap_or_else(|err| {
        log::error!(
            "Failed to load the embedded syntax set: {err}. Falling back to syntect defaults."
        );
        SyntaxSet::load_defaults_newlines()
    })
});

/// Get the extended syntax set.
pub fn load_full_syntax_set() -> &'static SyntaxSet {
    &SYNTAX_SET
}
