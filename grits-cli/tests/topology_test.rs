#[cfg(not(target_arch = "wasm32"))]
use grits_core::topology::parser::CodeParser;
use grits_core::topology::{analysis::TopologicalAnalysis, Symbol, SymbolGraph};

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
        package: None,
    });
    graph.add_symbol(grits_core::topology::Symbol {
        id: "B".to_string(),
        name: "B".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
        package: None,
    });
    graph.add_symbol(grits_core::topology::Symbol {
        id: "C".to_string(),
        name: "C".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
        package: None,
    });
    graph.add_symbol(grits_core::topology::Symbol {
        id: "D".to_string(),
        name: "D".to_string(),
        file_path: "f1".to_string(),
        language: "rust".to_string(),
        kind: "fn".to_string(),
        package: None,
    });

    let analysis = TopologicalAnalysis::analyze(&graph);

    // Nodes=4, Edges=4, Components=1, Triangles=0
    // Cycles = E - V + C - T = 4 - 4 + 1 - 0 = 1
    assert_eq!(analysis.betti_1, 1);
    assert_eq!(analysis.betti_0, 1);
    assert_eq!(analysis.triangle_count, 0);
}

#[test]
fn test_betti_2_octahedron() {
    // A hollow octahedron: 6 nodes, 12 edges, 8 triangles, 0 tetrahedra
    // χ = 6 - 12 + 8 - 0 = 2
    // β0 = 1, β1 = 0, β2 = 1
    let mut graph = SymbolGraph::new();
    for id in ["1", "2", "3", "4", "5", "6"] {
        graph.add_symbol(Symbol {
            id: id.to_string(),
            name: id.to_string(),
            file_path: "f".to_string(),
            language: "r".to_string(),
            kind: "k".to_string(),
            package: None,
        });
    }

    // Edges (1,2), (1,3), (1,4), (1,5)
    graph.add_dependency("1", "2", "calls");
    graph.add_dependency("1", "3", "calls");
    graph.add_dependency("1", "4", "calls");
    graph.add_dependency("1", "5", "calls");
    // Edges (6,2), (6,3), (6,4), (6,5)
    graph.add_dependency("6", "2", "calls");
    graph.add_dependency("6", "3", "calls");
    graph.add_dependency("6", "4", "calls");
    graph.add_dependency("6", "5", "calls");
    // Ring: (2,3), (3,4), (4,5), (5,2)
    graph.add_dependency("2", "3", "calls");
    graph.add_dependency("3", "4", "calls");
    graph.add_dependency("4", "5", "calls");
    graph.add_dependency("5", "2", "calls");

    let analysis = TopologicalAnalysis::analyze(&graph);

    println!("Betti 0: {}", analysis.betti_0);
    println!("Betti 1: {}", analysis.betti_1);
    println!("Betti 2: {}", analysis.betti_2);
    println!("Triangles: {}", analysis.triangle_count);

    assert_eq!(analysis.betti_0, 1);
    assert_eq!(analysis.triangle_count, 8);
    assert_eq!(analysis.betti_1, 0);
    assert_eq!(analysis.betti_2, 1);
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
    // Simulate File A -> B -> C -> A
    let code_a = r#"
        use file_b;
        fn a() { }
    "#;
    let code_b = r#"
        use file_c;
        fn b() { }
    "#;
    let code_c = r#"
        use file_a;
        fn c() { }
    "#;

    let mut graph = SymbolGraph::new();
    let mut parser = CodeParser::new("rust").expect("Failed to create parser");

    parser.parse_file("file_a", code_a, &mut graph).unwrap();
    parser.parse_file("file_b", code_b, &mut graph).unwrap();
    parser.parse_file("file_c", code_c, &mut graph).unwrap();

    let analysis = TopologicalAnalysis::analyze(&graph);

    // A 3rd-order cycle is a 2-simplex (triangle).
    // In our model, triangles are "filled", so Betti_1 might be 0, but triangle_count should be 1.
    assert_eq!(
        analysis.triangle_count, 1,
        "Should detect 1 triangle (solid feature)"
    );
}
