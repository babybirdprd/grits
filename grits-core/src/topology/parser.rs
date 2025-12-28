#[cfg(not(target_arch = "wasm32"))]
use super::{Symbol, SymbolGraph};
#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use streaming_iterator::StreamingIterator;
#[cfg(not(target_arch = "wasm32"))]
use tree_sitter::{Parser, Query, QueryCursor};

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
            language: self.language.clone(),
            kind: "file".to_string(),
        });

        let content_bytes = content.as_bytes();

        // For Rust: function_item, impl_item, struct_item, mod_item
        if self.language == "rust" {
            let query_str = r#"
                (function_item name: (identifier) @name) @func
                (struct_item name: (type_identifier) @name) @struct
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
                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                language: self.language.clone(),
                                kind: capture.node.kind().to_string(),
                            });
                            // Also link symbol to file
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "mod_name" {
                            graph.add_weighted_dependency(file_path, text, "imports", 0.3);
                        } else if capture_name == "call" {
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
                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                language: self.language.clone(),
                                kind: capture.node.kind().to_string(),
                            });
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "call" {
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
                            graph.add_symbol(Symbol {
                                id: id.clone(),
                                name: text.to_string(),
                                file_path: file_path.to_string(),
                                language: self.language.clone(),
                                kind: capture.node.kind().to_string(),
                            });
                            graph.add_weighted_dependency(&id, file_path, "defined_in", 1.0);
                        } else if capture_name == "call" {
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

        Ok(())
    }
}
