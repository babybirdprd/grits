#[cfg(not(target_arch = "wasm32"))]
use grits_core::topology::parser::CodeParser;
use grits_core::topology::SymbolGraph;

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_parser_integration() {
    let code = r#"
        use std::io;
        fn main() {
            println!("Hello");
            my_func();
        }
        struct MyStruct {
            field: i32
        }
    "#;

    let mut graph = SymbolGraph::new();
    let mut parser = CodeParser::new("rust").expect("Failed to create parser");
    parser
        .parse_file("test.rs", code, &mut graph)
        .expect("Failed to parse");

    // Should find main and MyStruct
    let found_main = graph.nodes.values().any(|s| s.name == "main");
    let found_struct = graph.nodes.values().any(|s| s.name == "MyStruct");

    assert!(found_main, "Should find main function");
    assert!(found_struct, "Should find MyStruct");

    // Should find imports and calls
    let found_import = graph
        .edges
        .iter()
        .any(|(_, to, e)| to.contains("io") && e.relation == "imports");
    let found_call = graph
        .edges
        .iter()
        .any(|(_, to, e)| to == "my_func" && e.relation == "calls");

    assert!(found_import, "Should find import");
    assert!(found_call, "Should find call to my_func");
}