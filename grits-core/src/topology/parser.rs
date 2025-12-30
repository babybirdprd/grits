#[cfg(not(target_arch = "wasm32"))]
use super::{Symbol, SymbolGraph};
#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use streaming_iterator::StreamingIterator;
#[cfg(not(target_arch = "wasm32"))]
use tree_sitter::{Parser, Query, QueryCursor};

/// Language built-ins that should be excluded from topology analysis.
/// These create "noise" in cycle detection as they're used everywhere.
#[cfg(not(target_arch = "wasm32"))]
fn is_builtin_symbol(name: &str, language: &str) -> bool {
    // Quick pattern-based checks (before expensive array lookups)
    // Filter single-letter identifiers (generic type params like T, E, F)
    if name.len() == 1
        && name
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_uppercase())
    {
        return true;
    }

    // Filter compound expressions that got captured (e.g., "Ok(())")
    if name.contains("()") || name.contains("::") {
        return true;
    }

    // Filter numeric literals and simple strings
    if name.parse::<f64>().is_ok() || name.starts_with('"') || name.starts_with('\'') {
        return true;
    }

    // Rust built-ins
    static RUST_BUILTINS: &[&str] = &[
        // Result/Option variants
        "Ok",
        "Err",
        "Some",
        "None",
        "Result",
        "Option",
        // Collections
        "String",
        "Vec",
        "Box",
        "Rc",
        "Arc",
        "RefCell",
        "Cell",
        "Mutex",
        "RwLock",
        "HashMap",
        "HashSet",
        "BTreeMap",
        "BTreeSet",
        "VecDeque",
        "LinkedList",
        "BinaryHeap",
        // Printing/debugging macros
        "println",
        "print",
        "eprintln",
        "eprint",
        "format",
        "panic",
        "assert",
        "assert_eq",
        "assert_ne",
        "debug_assert",
        "debug_assert_eq",
        "unreachable",
        "todo",
        "unimplemented",
        "dbg",
        "vec",
        "write",
        "writeln",
        "include_str",
        "include_bytes",
        "env",
        "concat",
        "stringify",
        "cfg",
        "matches",
        "log",
        "trace",
        "info",
        "warn",
        "error",
        // Common traits
        "Clone",
        "Copy",
        "Default",
        "Debug",
        "Display",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
        "Send",
        "Sync",
        "Sized",
        "Drop",
        "From",
        "Into",
        "TryFrom",
        "TryInto",
        "AsRef",
        "AsMut",
        "Deref",
        "DerefMut",
        "Iterator",
        "IntoIterator",
        "Extend",
        "FromIterator",
        "Read",
        "Write",
        "Seek",
        "BufRead",
        "Error",
        "Serialize",
        "Deserialize",
        "Fn",
        "FnMut",
        "FnOnce",
        // Primitive types
        "()",
        "unit",
        "bool",
        "true",
        "false",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "f32",
        "f64",
        "str",
        "char",
        // Keywords and common identifiers
        "Self",
        "self",
        "static",
        "mut",
        "ref",
        "pub",
        "move",
        "drop",
        "forget",
        "new",
        "default",
        // Error handling (anyhow, thiserror, etc.)
        "anyhow",
        "bail",
        "ensure",
        "context",
        "Context",
        "with_context",
        // Path and filesystem (commonly imported everywhere)
        "Path",
        "PathBuf",
        "File",
        "fs",
        "io",
        "std",
        // Common method names that get captured as calls
        "clone",
        "into",
        "from",
        "as_ref",
        "as_mut",
        "unwrap",
        "unwrap_or",
        "unwrap_or_else",
        "unwrap_or_default",
        "expect",
        "ok",
        "err",
        "is_ok",
        "is_err",
        "is_some",
        "is_none",
        "map",
        "map_err",
        "and_then",
        "or_else",
        "take",
        "replace",
        "get",
        "get_mut",
        "insert",
        "remove",
        "contains",
        "push",
        "pop",
        "len",
        "is_empty",
        "iter",
        "iter_mut",
        "into_iter",
        "collect",
        "filter",
        "find",
        "any",
        "all",
        "to_string",
        "to_owned",
        "as_str",
        "as_bytes",
        "parse",
        "try_into",
        "try_from",
        // Async runtime
        "async",
        "await",
        "spawn",
        "block_on",
        "tokio",
        "async_std",
    ];

    // TypeScript/JavaScript built-ins
    static JS_BUILTINS: &[&str] = &[
        "console",
        "log",
        "error",
        "warn",
        "info",
        "debug",
        "Promise",
        "async",
        "await",
        "setTimeout",
        "setInterval",
        "clearTimeout",
        "clearInterval",
        "Array",
        "Object",
        "String",
        "Number",
        "Boolean",
        "Symbol",
        "Map",
        "Set",
        "WeakMap",
        "WeakSet",
        "JSON",
        "Math",
        "Date",
        "RegExp",
        "Error",
        "TypeError",
        "RangeError",
        "SyntaxError",
        "parseInt",
        "parseFloat",
        "isNaN",
        "isFinite",
        "encodeURI",
        "decodeURI",
        "fetch",
        "require",
        "module",
        "exports",
        "process",
    ];

    // Python built-ins
    static PYTHON_BUILTINS: &[&str] = &[
        "print",
        "len",
        "range",
        "str",
        "int",
        "float",
        "bool",
        "list",
        "dict",
        "set",
        "tuple",
        "open",
        "input",
        "type",
        "isinstance",
        "hasattr",
        "getattr",
        "setattr",
        "delattr",
        "sum",
        "min",
        "max",
        "abs",
        "round",
        "sorted",
        "reversed",
        "enumerate",
        "zip",
        "map",
        "filter",
        "any",
        "all",
        "next",
        "iter",
        "super",
        "property",
        "classmethod",
        "staticmethod",
        "Exception",
        "ValueError",
        "TypeError",
        "KeyError",
        "IndexError",
        "AttributeError",
    ];

    // Go built-ins
    static GO_BUILTINS: &[&str] = &[
        "fmt", "Println", "Printf", "Sprintf", "Errorf", "Print", "make", "new", "len", "cap",
        "append", "copy", "delete", "close", "panic", "recover", "error", "nil",
    ];

    match language {
        "rust" => RUST_BUILTINS.contains(&name),
        "typescript" | "ts" | "javascript" | "js" => JS_BUILTINS.contains(&name),
        "python" | "py" => PYTHON_BUILTINS.contains(&name),
        "go" => GO_BUILTINS.contains(&name),
        _ => false,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct CodeParser {
    parser: Parser,
    language: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl CodeParser {
    pub fn new(language: &str) -> Result<Self> {
        let mut parser = Parser::new();
        let lang: tree_sitter::Language = match language {
            "rust" => tree_sitter_rust::LANGUAGE.into(),
            "typescript" | "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "javascript" | "js" => tree_sitter_javascript::LANGUAGE.into(),
            "python" | "py" => tree_sitter_python::LANGUAGE.into(),
            "go" => tree_sitter_go::LANGUAGE.into(),
            _ => return Err(anyhow::anyhow!("Unsupported language: {}", language)),
        };

        parser.set_language(&lang)?;

        Ok(Self {
            parser,
            language: language.to_string(),
        })
    }

    pub fn parse_file(
        &mut self,
        file_path: &str,
        content: &str,
        graph: &mut SymbolGraph,
    ) -> Result<()> {
        let tree = self
            .parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse"))?;
        let root = tree.root_node();

        // 1. Add "File Node" to ensure connectivity for top-level dependencies (imports)
        graph.add_symbol(Symbol {
            id: file_path.to_string(),
            name: file_path.to_string(),
            file_path: file_path.to_string(),
            package: None,
            language: self.language.clone(),
            kind: "file".to_string(),
            byte_range: Some((0, content.len())),
            metadata: HashMap::new(),
        });

        let content_bytes = content.as_bytes();

        // For Rust: function_item, impl_item, struct_item, mod_item
        if self.language == "rust" {
            let query_str = r#"
                (function_item name: (identifier) @name) @func
                (struct_item name: (type_identifier) @name) @struct
                (impl_item type: (type_identifier) @name) @impl
                (mod_item name: (identifier) @mod_name) @mod
                (use_declaration argument: (_) @import) @use
                (call_expression function: (identifier) @call) @call
             "#;
            let lang: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
            if let Ok(query) = Query::new(&lang, query_str) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, root, content_bytes);
                while let Some(match_) = matches.next() {
                    for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name: &str = &query.capture_names()[idx];
                        let range = capture.node.byte_range();
                        if range.end > content.len() {
                            continue;
                        }
                        let text = &content[range.start..range.end];

                        if capture_name == "name" {
                            let id = format!("{}::{}", file_path, text);
                            let kind = capture.node.kind().to_string();
                            let mut decl_node = capture.node;
                            if let Some(parent) = decl_node.parent() {
                                let pk = parent.kind();
                                if pk.contains("_item")
                                    || pk == "function_item"
                                    || pk == "struct_item"
                                    || pk == "impl_item"
                                {
                                    decl_node = parent;
                                }
                            }
                            let full_range = decl_node.byte_range();

                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                package: None,
                                language: self.language.clone(),
                                kind: kind.clone(),
                                byte_range: Some((full_range.start, full_range.end)),
                                metadata: HashMap::new(),
                            });
                            // Also link symbol to file
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);

                            // If this is a function, check if it's inside an impl block
                            if kind == "function_item"
                                || capture
                                    .node
                                    .parent()
                                    .map_or(false, |p| p.kind() == "function_item")
                            {
                                let mut curr = capture.node.parent();
                                while let Some(p) = curr {
                                    if p.kind() == "impl_item" {
                                        if let Some(type_node) = p.child_by_field_name("type") {
                                            let type_range = type_node.byte_range();
                                            if type_range.end <= content.len() {
                                                let struct_name =
                                                    &content[type_range.start..type_range.end];
                                                let struct_id =
                                                    format!("{}::{}", file_path, struct_name);
                                                // Link function to struct via "implemented_for"
                                                graph.add_weighted_dependency(
                                                    &id, &struct_id, "part_of", 1.0,
                                                );
                                            }
                                        }
                                        break;
                                    }
                                    curr = p.parent();
                                }
                            }
                        } else if capture_name == "mod_name" {
                            graph.add_weighted_dependency(file_path, text, "imports", 0.3);
                        } else if capture_name == "call" {
                            // Skip language built-ins that create noise in cycle detection
                            if is_builtin_symbol(text, &self.language) {
                                continue;
                            }
                            let mut strength = 0.6;
                            let mut parent = capture.node.parent();
                            while let Some(p) = parent {
                                let kind = p.kind();
                                if kind.contains("loop")
                                    || kind == "for_expression"
                                    || kind == "while_expression"
                                {
                                    strength = 1.0;
                                    break;
                                }
                                parent = p.parent();
                            }
                            graph.add_weighted_dependency(file_path, text, "calls", strength);
                        } else if capture_name == "import" {
                            let import_path = text.trim();
                            if import_path.starts_with("crate::") {
                                let parts: Vec<&str> = import_path.split("::").collect();
                                if parts.len() >= 2 {
                                    graph.add_weighted_dependency(
                                        file_path, parts[1], "imports", 0.3,
                                    );
                                }
                            } else {
                                graph.add_weighted_dependency(
                                    file_path,
                                    import_path,
                                    "imports",
                                    0.3,
                                );
                            }
                        }
                    }
                }
            }
        }

        // For TypeScript
        if self.language == "typescript" || self.language == "ts" {
            let query_str = r#"
                (function_declaration name: (identifier) @name) @func
                (class_declaration name: (type_identifier) @name) @class
                (import_statement source: (string) @import) @import
                (call_expression function: (identifier) @call) @call
             "#;
            let lang: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
            if let Ok(query) = Query::new(&lang, query_str) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, root, content_bytes);
                while let Some(match_) = matches.next() {
                    for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name: &str = &query.capture_names()[idx];
                        let range = capture.node.byte_range();
                        if range.end > content.len() {
                            continue;
                        }
                        let text = &content[range.start..range.end];

                        if capture_name == "name" {
                            let id = format!("{}::{}", file_path, text);
                            let kind = capture.node.kind().to_string();
                            let mut decl_node = capture.node;
                            if let Some(parent) = decl_node.parent() {
                                let pk = parent.kind();
                                if pk.contains("declaration") || pk.contains("statement") {
                                    decl_node = parent;
                                }
                            }
                            let full_range = decl_node.byte_range();

                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                package: None,
                                language: self.language.clone(),
                                kind,
                                byte_range: Some((full_range.start, full_range.end)),
                                metadata: HashMap::new(),
                            });
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "call" {
                            // Skip language built-ins that create noise in cycle detection
                            if is_builtin_symbol(text, &self.language) {
                                continue;
                            }
                            let mut strength = 0.6;
                            let mut parent = capture.node.parent();
                            while let Some(p) = parent {
                                let kind = p.kind();
                                if kind.contains("Statement")
                                    && (kind.contains("For")
                                        || kind.contains("While")
                                        || kind.contains("Do"))
                                {
                                    strength = 1.0;
                                    break;
                                }
                                parent = p.parent();
                            }
                            graph.add_weighted_dependency(file_path, text, "calls", strength);
                        } else if capture_name == "import" {
                            let clean_import = text.trim_matches(|c| c == '\'' || c == '"');
                            graph.add_weighted_dependency(file_path, clean_import, "imports", 0.3);
                        }
                    }
                }
            }
        }

        // For JavaScript (ES6 imports)
        if self.language == "javascript" || self.language == "js" {
            let query_str = r#"
                (function_declaration name: (identifier) @name) @func
                (class_declaration name: (identifier) @name) @class
                (import_statement source: (string) @import) @import
                (call_expression function: (identifier) @call) @call
             "#;
            let lang: tree_sitter::Language = tree_sitter_javascript::LANGUAGE.into();
            if let Ok(query) = Query::new(&lang, query_str) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, root, content_bytes);
                while let Some(match_) = matches.next() {
                    for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name: &str = &query.capture_names()[idx];
                        let range = capture.node.byte_range();
                        if range.end > content.len() {
                            continue;
                        }
                        let text = &content[range.start..range.end];

                        if capture_name == "name" {
                            let id = format!("{}::{}", file_path, text);
                            let kind = capture.node.kind().to_string();
                            let mut decl_node = capture.node;
                            if let Some(parent) = decl_node.parent() {
                                let pk = parent.kind();
                                if pk.contains("declaration") || pk.contains("statement") {
                                    decl_node = parent;
                                }
                            }
                            let full_range = decl_node.byte_range();

                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                package: None,
                                language: self.language.clone(),
                                kind,
                                byte_range: Some((full_range.start, full_range.end)),
                                metadata: HashMap::new(),
                            });
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "call" {
                            // Skip language built-ins that create noise in cycle detection
                            if is_builtin_symbol(text, &self.language) {
                                continue;
                            }
                            let mut strength = 0.6;
                            let mut parent = capture.node.parent();
                            while let Some(p) = parent {
                                let kind = p.kind();
                                if kind.contains("Statement")
                                    && (kind.contains("For")
                                        || kind.contains("While")
                                        || kind.contains("Do"))
                                {
                                    strength = 1.0;
                                    break;
                                }
                                parent = p.parent();
                            }
                            graph.add_weighted_dependency(file_path, text, "calls", strength);
                        } else if capture_name == "import" {
                            let clean_import = text.trim_matches(|c| c == '\'' || c == '"');
                            graph.add_weighted_dependency(file_path, clean_import, "imports", 0.3);
                        }
                    }
                }
            }
        }

        // For Python
        if self.language == "python" || self.language == "py" {
            let query_str = r#"
                (function_definition name: (identifier) @name) @func
                (class_definition name: (identifier) @name) @class
                (import_statement name: (dotted_name) @import) @import
                (import_from_statement module_name: (dotted_name) @import) @import_from
                (call function: (identifier) @call) @call
             "#;
            let lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
            if let Ok(query) = Query::new(&lang, query_str) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, root, content_bytes);
                while let Some(match_) = matches.next() {
                    for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name: &str = &query.capture_names()[idx];
                        let range = capture.node.byte_range();
                        if range.end > content.len() {
                            continue;
                        }
                        let text = &content[range.start..range.end];

                        if capture_name == "name" {
                            let id = format!("{}::{}", file_path, text);
                            let kind = capture.node.kind().to_string();
                            let mut decl_node = capture.node;
                            if let Some(parent) = decl_node.parent() {
                                let pk = parent.kind();
                                if pk.contains("definition") || pk.contains("statement") {
                                    decl_node = parent;
                                }
                            }
                            let full_range = decl_node.byte_range();

                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                package: None,
                                language: self.language.clone(),
                                kind,
                                byte_range: Some((full_range.start, full_range.end)),
                                metadata: HashMap::new(),
                            });
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "call" {
                            // Skip language built-ins that create noise in cycle detection
                            if is_builtin_symbol(text, &self.language) {
                                continue;
                            }
                            let mut strength = 0.6;
                            let mut parent = capture.node.parent();
                            while let Some(p) = parent {
                                let kind = p.kind();
                                if kind == "for_statement" || kind == "while_statement" {
                                    strength = 1.0;
                                    break;
                                }
                                parent = p.parent();
                            }
                            graph.add_weighted_dependency(file_path, text, "calls", strength);
                        } else if capture_name == "import" {
                            graph.add_weighted_dependency(file_path, text, "imports", 0.3);
                        }
                    }
                }
            }
        }

        // For Go
        if self.language == "go" {
            let query_str = r#"
                (function_declaration name: (identifier) @name) @func
                (method_declaration name: (field_identifier) @name) @method
                (type_declaration (type_spec name: (type_identifier) @name)) @type
                (import_spec path: (interpreted_string_literal) @import) @import
                (call_expression function: (identifier) @call) @call
             "#;
            let lang: tree_sitter::Language = tree_sitter_go::LANGUAGE.into();
            if let Ok(query) = Query::new(&lang, query_str) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, root, content_bytes);
                while let Some(match_) = matches.next() {
                    for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name: &str = &query.capture_names()[idx];
                        let range = capture.node.byte_range();
                        if range.end > content.len() {
                            continue;
                        }
                        let text = &content[range.start..range.end];

                        if capture_name == "name" {
                            let id = format!("{}::{}", file_path, text);
                            let kind = capture.node.kind().to_string();
                            let mut decl_node = capture.node;
                            if let Some(parent) = decl_node.parent() {
                                let pk = parent.kind();
                                if pk.contains("declaration") || pk.contains("spec") {
                                    decl_node = parent;
                                }
                            }
                            let full_range = decl_node.byte_range();

                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                package: None,
                                language: self.language.clone(),
                                kind,
                                byte_range: Some((full_range.start, full_range.end)),
                                metadata: HashMap::new(),
                            });
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "call" {
                            // Skip language built-ins that create noise in cycle detection
                            if is_builtin_symbol(text, &self.language) {
                                continue;
                            }
                            let mut strength = 0.6;
                            let mut parent = capture.node.parent();
                            while let Some(p) = parent {
                                let kind = p.kind();
                                if kind == "for_statement" || kind == "range_clause" {
                                    strength = 1.0;
                                    break;
                                }
                                parent = p.parent();
                            }
                            graph.add_weighted_dependency(file_path, text, "calls", strength);
                        } else if capture_name == "import" {
                            let clean_import = text.trim_matches('"');
                            graph.add_weighted_dependency(file_path, clean_import, "imports", 0.3);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
