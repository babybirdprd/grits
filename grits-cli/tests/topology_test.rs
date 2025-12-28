#[cfg(not(target_arch = "wasm32"))]
use grits_core::topology::parser::CodeParser;
use grits_core::topology::{analysis::TopologicalAnalysis, SymbolGraph};

#[test]
fn test_topology_analysis() {
    let mut graph = SymbolGraph::new();

    // Create a 4-node cycle: A -> B -> C -> D -> A (this is a hole, not a filled simplex)
    // A 3-node cycle (triangle) would be a 2-simplex (filled), so betti_1=0
    // A 4-node cycle is a true 1-cycle (hole), so betti_1=1
    graph.add_dependency("A", "B", "calls");
    graph.add_dependency("B", "C", "calls");
    graph.add_dependency("C", "D", "calls");
    graph.add_dependency("D", "A", "calls");

    // Add nodes
    graph.add_symbol(grits_core::topology::Symbol {
        id: "A".to_string(),
        name: "A".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
    });
    graph.add_symbol(grits_core::topology::Symbol {
        id: "B".to_string(),
        name: "B".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
    });
    graph.add_symbol(grits_core::topology::Symbol {
        id: "C".to_string(),
        name: "C".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
    });
    graph.add_symbol(grits_core::topology::Symbol {
        id: "D".to_string(),
        name: "D".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
    });

    let analysis = TopologicalAnalysis::analyze(&graph);

    // Nodes=4, Edges=4, Components=1, Triangles=0
    // Cycles = E - V + C - T = 4 - 4 + 1 - 0 = 1
    assert_eq!(analysis.betti_1, 1);
    assert_eq!(analysis.betti_0, 1);
    assert_eq!(analysis.triangle_count, 0);
}

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

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_cycle_detection_via_parser() {
    // Simulate File A importing B, and B importing A
    let code_a = r#"
        use file_b;
        fn a() { }
    "#;
    let code_b = r#"
        use file_a;
        fn b() { }
    "#;

    let mut graph = SymbolGraph::new();
    let mut parser = CodeParser::new("rust").expect("Failed to create parser");

    // Parse File A (ID: "file_a")
    parser
        .parse_file("file_a", code_a, &mut graph)
        .expect("Failed to parse A");
    // Parse File B (ID: "file_b")
    parser
        .parse_file("file_b", code_b, &mut graph)
        .expect("Failed to parse B");

    // Check connections
    // A -> file_b (import)
    // B -> file_a (import)
    // With implicit nodes handled, we have:
    // Nodes: file_a, file_b, "file_a" (from import string), "file_b" (from import string)
    // Wait, the import text is "file_b", and the file ID is "file_b".
    // So they should match!

    let analysis = TopologicalAnalysis::analyze(&graph);

    // We expect a cycle: file_a -> file_b -> file_a
    assert!(
        analysis.betti_1 >= 1,
        "Should detect at least 1 cycle (Betti_1 >= 1), found {}",
        analysis.betti_1
    );
}
