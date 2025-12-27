#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use tree_sitter::{Parser, Query, QueryCursor};
#[cfg(not(target_arch = "wasm32"))]
use super::{Symbol, SymbolGraph};

#[cfg(not(target_arch = "wasm32"))]
pub struct CodeParser {
    parser: Parser,
    language: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl CodeParser {
    pub fn new(language: &str) -> Result<Self> {
        let mut parser = Parser::new();
        let lang = match language {
            "rust" => tree_sitter_rust::language(),
            "typescript" | "ts" => tree_sitter_typescript::language_typescript(),
            "javascript" | "js" => tree_sitter_javascript::language(),
            _ => return Err(anyhow::anyhow!("Unsupported language: {}", language)),
        };

        parser.set_language(lang)?;

        Ok(Self {
            parser,
            language: language.to_string(),
        })
    }

    pub fn parse_file(&mut self, file_path: &str, content: &str, graph: &mut SymbolGraph) -> Result<()> {
        let tree = self.parser.parse(content, None).ok_or_else(|| anyhow::anyhow!("Failed to parse"))?;
        let root = tree.root_node();

        // 1. Add "File Node" to ensure connectivity for top-level dependencies (imports)
        graph.add_symbol(Symbol {
            id: file_path.to_string(),
            name: file_path.to_string(),
            file_path: file_path.to_string(),
            language: self.language.clone(),
            kind: "file".to_string(),
        });

        // For Rust: function_item, impl_item, struct_item
        if self.language == "rust" {
             let query_str = r#"
                (function_item name: (identifier) @name) @func
                (struct_item name: (type_identifier) @name) @struct
                (use_declaration argument: (_) @import) @use
                (call_expression function: (identifier) @call) @call
             "#;
             if let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) {
                 let mut cursor = QueryCursor::new();
                 for match_ in cursor.matches(&query, root, content.as_bytes()) {
                     for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name = query.capture_names()[idx].as_str();
                        let range = capture.node.byte_range();
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
                             graph.add_dependency(&id, file_path, "defined_in");
                        } else if capture_name == "call" {
                             // Heuristic: call to X depends on X
                             // We link from the FILE for now (simplification)
                             graph.add_dependency(file_path, text, "calls");
                        } else if capture_name == "import" {
                             // "use std::io;" -> text is "std::io"
                             // We link File -> Import
                             graph.add_dependency(file_path, text, "imports");
                        }
                     }
                 }
             }
        }

        // For TS
        if self.language == "typescript" || self.language == "ts" {
             let query_str = r#"
                (function_declaration name: (identifier) @name) @func
                (class_declaration name: (type_identifier) @name) @class
                (import_statement source: (string) @import) @import
                (call_expression function: (identifier) @call) @call
             "#;
             if let Ok(query) = Query::new(tree_sitter_typescript::language_typescript(), query_str) {
                 let mut cursor = QueryCursor::new();
                 for match_ in cursor.matches(&query, root, content.as_bytes()) {
                     for capture in match_.captures {
                        let idx = capture.index as usize;
                        let capture_name = query.capture_names()[idx].as_str();
                        let range = capture.node.byte_range();
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
                             graph.add_dependency(&id, file_path, "defined_in");
                        } else if capture_name == "call" {
                             graph.add_dependency(file_path, text, "calls");
                        } else if capture_name == "import" {
                             // Remove quotes
                             let clean_import = text.trim_matches(|c| c == '\'' || c == '"');
                             graph.add_dependency(file_path, clean_import, "imports");
                        }
                     }
                 }
             }
        }

        Ok(())
    }
}
