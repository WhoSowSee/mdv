use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn expand_language_aliases(token: &str) -> Vec<String> {
        let mut aliases = Vec::new();
        Self::push_candidate(&mut aliases, token);

        let lower = token.to_lowercase();
        if lower != token {
            Self::push_candidate(&mut aliases, &lower);
        }

        match lower.as_str() {
            "rs" | "rust" => {
                Self::push_candidate(&mut aliases, "rs");
                Self::push_candidate(&mut aliases, "rust");
                Self::push_candidate(&mut aliases, "Rust");
            }
            "py" | "python" => {
                Self::push_candidate(&mut aliases, "py");
                Self::push_candidate(&mut aliases, "python");
                Self::push_candidate(&mut aliases, "Python");
            }
            "js" | "javascript" | "node" | "nodejs" | "ecmascript" => {
                Self::push_candidate(&mut aliases, "js");
                Self::push_candidate(&mut aliases, "javascript");
                Self::push_candidate(&mut aliases, "JavaScript");
                Self::push_candidate(&mut aliases, "JavaScript (Babel)");
            }
            "jsx" => {
                Self::push_candidate(&mut aliases, "jsx");
                Self::push_candidate(&mut aliases, "JavaScript (Babel)");
            }
            "ts" | "typescript" => {
                Self::push_candidate(&mut aliases, "ts");
                Self::push_candidate(&mut aliases, "typescript");
                Self::push_candidate(&mut aliases, "TypeScript");
            }
            "tsx" | "typescriptreact" => {
                Self::push_candidate(&mut aliases, "tsx");
                Self::push_candidate(&mut aliases, "TypeScriptReact");
                Self::push_candidate(&mut aliases, "TypeScript");
            }
            "c" => {
                Self::push_candidate(&mut aliases, "c");
                Self::push_candidate(&mut aliases, "C");
            }
            "h" => {
                Self::push_candidate(&mut aliases, "c");
                Self::push_candidate(&mut aliases, "C");
            }
            "cpp" | "c++" | "cxx" | "hpp" => {
                Self::push_candidate(&mut aliases, "cpp");
                Self::push_candidate(&mut aliases, "c++");
                Self::push_candidate(&mut aliases, "C++");
                Self::push_candidate(&mut aliases, "cxx");
            }
            "objc" | "objective-c" | "objectivec" => {
                Self::push_candidate(&mut aliases, "objc");
                Self::push_candidate(&mut aliases, "Objective-C");
                Self::push_candidate(&mut aliases, "Objectivec");
            }
            "objcpp" | "objective-c++" => {
                Self::push_candidate(&mut aliases, "objective-c++");
                Self::push_candidate(&mut aliases, "Objective-C++");
                Self::push_candidate(&mut aliases, "objcpp");
            }
            "cs" | "csharp" | "c#" => {
                Self::push_candidate(&mut aliases, "cs");
                Self::push_candidate(&mut aliases, "csharp");
                Self::push_candidate(&mut aliases, "C#");
            }
            "go" | "golang" => {
                Self::push_candidate(&mut aliases, "go");
                Self::push_candidate(&mut aliases, "Go");
            }
            "java" => {
                Self::push_candidate(&mut aliases, "java");
                Self::push_candidate(&mut aliases, "Java");
            }
            "kotlin" | "kt" => {
                Self::push_candidate(&mut aliases, "kt");
                Self::push_candidate(&mut aliases, "kotlin");
                Self::push_candidate(&mut aliases, "Kotlin");
            }
            "swift" => {
                Self::push_candidate(&mut aliases, "swift");
                Self::push_candidate(&mut aliases, "Swift");
            }
            "scala" => {
                Self::push_candidate(&mut aliases, "scala");
                Self::push_candidate(&mut aliases, "Scala");
            }
            "php" => {
                Self::push_candidate(&mut aliases, "php");
                Self::push_candidate(&mut aliases, "PHP");
            }
            "rb" | "ruby" => {
                Self::push_candidate(&mut aliases, "rb");
                Self::push_candidate(&mut aliases, "ruby");
                Self::push_candidate(&mut aliases, "Ruby");
            }
            "perl" | "pl" => {
                Self::push_candidate(&mut aliases, "pl");
                Self::push_candidate(&mut aliases, "Perl");
            }
            "lua" => {
                Self::push_candidate(&mut aliases, "lua");
                Self::push_candidate(&mut aliases, "Lua");
            }
            "r" => {
                Self::push_candidate(&mut aliases, "r");
                Self::push_candidate(&mut aliases, "R");
            }
            "dart" => {
                Self::push_candidate(&mut aliases, "dart");
                Self::push_candidate(&mut aliases, "Dart");
            }
            "haskell" | "hs" => {
                Self::push_candidate(&mut aliases, "hs");
                Self::push_candidate(&mut aliases, "Haskell");
            }
            "clj" | "clojure" => {
                Self::push_candidate(&mut aliases, "clj");
                Self::push_candidate(&mut aliases, "Clojure");
            }
            "elixir" => {
                Self::push_candidate(&mut aliases, "elixir");
                Self::push_candidate(&mut aliases, "Elixir");
            }
            "erlang" => {
                Self::push_candidate(&mut aliases, "erlang");
                Self::push_candidate(&mut aliases, "Erlang");
            }
            "fsharp" | "fs" | "f#" => {
                Self::push_candidate(&mut aliases, "F#");
                Self::push_candidate(&mut aliases, "FSharp");
                Self::push_candidate(&mut aliases, "fs");
            }
            "sql" | "sqlite" | "postgres" | "mysql" => {
                Self::push_candidate(&mut aliases, "sql");
                Self::push_candidate(&mut aliases, "SQL");
            }
            "yaml" | "yml" => {
                Self::push_candidate(&mut aliases, "yaml");
                Self::push_candidate(&mut aliases, "YAML");
                Self::push_candidate(&mut aliases, "yml");
            }
            "json" | "jsonc" | "json5" => {
                Self::push_candidate(&mut aliases, "json");
                Self::push_candidate(&mut aliases, "JSON");
            }
            "toml" => {
                Self::push_candidate(&mut aliases, "toml");
                Self::push_candidate(&mut aliases, "TOML");
            }
            "ini" | "cfg" | "conf" => {
                Self::push_candidate(&mut aliases, "ini");
                Self::push_candidate(&mut aliases, "INI");
            }
            "md" | "markdown" => {
                Self::push_candidate(&mut aliases, "md");
                Self::push_candidate(&mut aliases, "markdown");
                Self::push_candidate(&mut aliases, "Markdown");
            }
            "html" | "htm" | "xhtml" => {
                Self::push_candidate(&mut aliases, "html");
                Self::push_candidate(&mut aliases, "HTML");
            }
            "xml" => {
                Self::push_candidate(&mut aliases, "xml");
                Self::push_candidate(&mut aliases, "XML");
            }
            "css" => {
                Self::push_candidate(&mut aliases, "css");
                Self::push_candidate(&mut aliases, "CSS");
            }
            "scss" => {
                Self::push_candidate(&mut aliases, "scss");
                Self::push_candidate(&mut aliases, "SCSS");
            }
            "less" => {
                Self::push_candidate(&mut aliases, "less");
                Self::push_candidate(&mut aliases, "LESS");
            }
            "bash" | "sh" | "shell" | "zsh" | "shell-session" | "console" => {
                Self::push_candidate(&mut aliases, "bash");
                Self::push_candidate(&mut aliases, "Bash");
                Self::push_candidate(&mut aliases, "shell");
                Self::push_candidate(&mut aliases, "Shell");
                Self::push_candidate(&mut aliases, "Shell-Unix-Generic");
                Self::push_candidate(&mut aliases, "sh");
            }
            "fish" => {
                Self::push_candidate(&mut aliases, "fish");
                Self::push_candidate(&mut aliases, "Fish");
            }
            "powershell" | "ps" | "ps1" => {
                Self::push_candidate(&mut aliases, "powershell");
                Self::push_candidate(&mut aliases, "PowerShell");
                Self::push_candidate(&mut aliases, "ps1");
            }
            "cmd" | "batch" | "bat" => {
                Self::push_candidate(&mut aliases, "Batchfile");
                Self::push_candidate(&mut aliases, "batch");
                Self::push_candidate(&mut aliases, "bat");
            }
            "make" | "makefile" => {
                Self::push_candidate(&mut aliases, "make");
                Self::push_candidate(&mut aliases, "Makefile");
            }
            "cmake" => {
                Self::push_candidate(&mut aliases, "cmake");
                Self::push_candidate(&mut aliases, "CMake");
            }
            "docker" | "dockerfile" => {
                Self::push_candidate(&mut aliases, "docker");
                Self::push_candidate(&mut aliases, "Dockerfile");
            }
            "graphql" | "gql" => {
                Self::push_candidate(&mut aliases, "graphql");
                Self::push_candidate(&mut aliases, "GraphQL");
            }
            "proto" | "protobuf" => {
                Self::push_candidate(&mut aliases, "proto");
                Self::push_candidate(&mut aliases, "Protocol Buffer");
            }
            "plantuml" | "uml" => {
                Self::push_candidate(&mut aliases, "plantuml");
                Self::push_candidate(&mut aliases, "PlantUML");
            }
            "mermaid" => {
                Self::push_candidate(&mut aliases, "mermaid");
                Self::push_candidate(&mut aliases, "Mermaid");
            }
            "diff" | "patch" | "gdiff" => {
                Self::push_candidate(&mut aliases, "diff");
                Self::push_candidate(&mut aliases, "Diff");
                Self::push_candidate(&mut aliases, "patch");
            }
            "log" => {
                Self::push_candidate(&mut aliases, "Log");
            }
            "latex" | "tex" => {
                Self::push_candidate(&mut aliases, "latex");
                Self::push_candidate(&mut aliases, "LaTeX");
                Self::push_candidate(&mut aliases, "tex");
                Self::push_candidate(&mut aliases, "TeX");
            }
            "rst" | "restructuredtext" => {
                Self::push_candidate(&mut aliases, "rst");
                Self::push_candidate(&mut aliases, "reStructuredText");
            }
            "adoc" | "asciidoc" => {
                Self::push_candidate(&mut aliases, "adoc");
                Self::push_candidate(&mut aliases, "AsciiDoc");
            }
            "matlab" | "octave" => {
                Self::push_candidate(&mut aliases, "matlab");
                Self::push_candidate(&mut aliases, "Matlab");
                Self::push_candidate(&mut aliases, "Octave");
            }
            "vb" | "visualbasic" => {
                Self::push_candidate(&mut aliases, "vb");
                Self::push_candidate(&mut aliases, "Visual Basic");
                Self::push_candidate(&mut aliases, "VB.NET");
            }
            "zig" => {
                Self::push_candidate(&mut aliases, "zig");
                Self::push_candidate(&mut aliases, "Zig");
            }
            "nim" => {
                Self::push_candidate(&mut aliases, "nim");
                Self::push_candidate(&mut aliases, "Nim");
            }
            "solidity" | "sol" => {
                Self::push_candidate(&mut aliases, "solidity");
                Self::push_candidate(&mut aliases, "Solidity");
            }
            "proto3" => {
                Self::push_candidate(&mut aliases, "proto3");
                Self::push_candidate(&mut aliases, "Protocol Buffer");
            }
            "assembly" | "asm" => {
                Self::push_candidate(&mut aliases, "asm");
                Self::push_candidate(&mut aliases, "Assembly");
            }
            "wasm" | "wat" => {
                Self::push_candidate(&mut aliases, "wat");
                Self::push_candidate(&mut aliases, "WebAssembly");
            }
            _ => {}
        }

        aliases
    }

    pub(super) fn push_candidate(target: &mut Vec<String>, candidate: &str) {
        if candidate.is_empty() {
            return;
        }

        if target
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(candidate))
        {
            return;
        }

        target.push(candidate.to_string());
    }

    pub(super) fn is_plain_language(token: &str) -> bool {
        matches!(
            token.to_lowercase().as_str(),
            "text"
                | "plain"
                | "plaintext"
                | "plain_text"
                | "txt"
                | "output"
                | "nohighlight"
                | "none"
        )
    }
}
