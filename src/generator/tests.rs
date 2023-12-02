use {
    super::GraphGenerator,
    crate::lsp_types::{DocumentSymbol, Position, Range, SymbolKind},
};

#[test]
#[allow(deprecated)]
fn nested_function() {
    let mut generator = GraphGenerator::new("abc".to_string(), "");
    let parent_range = Range {
        start: Position {
            line: 1,
            character: 3,
        },
        end: Position {
            line: 1,
            character: 10,
        },
    };
    let child_range = Range {
        start: Position {
            line: 10,
            character: 4,
        },
        end: Position {
            line: 10,
            character: 16,
        },
    };

    generator.add_file(
        "abc".to_string(),
        vec![DocumentSymbol {
            name: "fn_parent".to_string(),
            detail: None,
            kind: SymbolKind::Function,
            tags: None,
            deprecated: None,
            range: parent_range,
            selection_range: parent_range,
            children: vec![DocumentSymbol {
                name: "fn_child".to_string(),
                detail: None,
                kind: SymbolKind::Function,
                tags: None,
                deprecated: None,
                range: child_range,
                selection_range: child_range,
                children: vec![],
            }],
        }],
    );

    let dot = generator.generate_dot_source();
    println!("{}", dot);
}
