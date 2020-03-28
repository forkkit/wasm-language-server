pub mod node {
    use lsp_types::*;
    use smol_str::SmolStr;
    use tree_sitter::Node;

    pub mod position {
        use lsp_types::*;
        use tree_sitter::{Node, Point};

        pub fn start(node: &Node) -> Position {
            let Point { row, column } = node.start_position();
            Position::new(row as u64, column as u64)
        }

        pub fn end(node: &Node) -> Position {
            let Point { row, column } = node.end_position();
            Position::new(row as u64, column as u64)
        }
    }

    pub fn range(node: &Node) -> Range {
        Range::new(position::start(node), position::end(node))
    }

    #[derive(Clone, Debug)]
    pub struct NameAndRanges {
        pub name: SmolStr,
        pub range: Range,
        pub selection_range: Range,
    }

    pub fn name_and_ranges<'a>(source: &'a [u8], node: &Node, outer_id: u16, inner_id: Option<u16>) -> NameAndRanges {
        let name;
        let range = crate::lsp::node::range(&node);
        let selection_range;
        if let Some(outer_node) = node.child_by_field_id(outer_id) {
            let inner_node = if let Some(inner_id) = inner_id {
                outer_node.child_by_field_id(inner_id).unwrap()
            } else {
                outer_node
            };
            name = SmolStr::new(inner_node.utf8_text(source).unwrap());
            selection_range = crate::lsp::node::range(&inner_node);
        } else {
            name = SmolStr::new("<anonymous>");
            selection_range = range;
        }

        NameAndRanges {
            name,
            range,
            selection_range,
        }
    }
}
