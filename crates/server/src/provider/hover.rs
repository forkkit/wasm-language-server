//! Provides `textDocument/hover` functionality.

use crate::core::{
    self,
    language::{wast, wat},
};
use std::sync::Arc;

/// Compute "textDocument/hover" for a given document.
pub async fn response(session: Arc<core::Session>, params: lsp::HoverParams) -> anyhow::Result<Option<lsp::Hover>> {
    let document = session
        .get_document(&params.text_document_position_params.text_document.uri)
        .await?;
    let range = lsp::Range::new(
        params.text_document_position_params.position,
        params.text_document_position_params.position,
    );

    let module_fields: &[u16] = &[
        *wast::kind::MODULE_FIELD_DATA,
        *wast::kind::MODULE_FIELD_ELEM,
        *wast::kind::MODULE_FIELD_FUNC,
        *wast::kind::MODULE_FIELD_GLOBAL,
        *wast::kind::MODULE_FIELD_MEMORY,
        *wast::kind::MODULE_FIELD_TABLE,
        *wast::kind::MODULE_FIELD_TYPE,
        *wat::kind::MODULE_FIELD_DATA,
        *wat::kind::MODULE_FIELD_ELEM,
        *wat::kind::MODULE_FIELD_FUNC,
        *wat::kind::MODULE_FIELD_GLOBAL,
        *wat::kind::MODULE_FIELD_MEMORY,
        *wat::kind::MODULE_FIELD_TABLE,
        *wat::kind::MODULE_FIELD_TYPE,
    ];

    let start = crate::util::position::byte_index(&document, &range.start)?;
    let end = crate::util::position::byte_index(&document, &range.end)?;

    let mut contents = vec![];
    let mut range = None;

    let tree = session
        .get_tree(&params.text_document_position_params.text_document.uri)
        .await?;
    let tree = tree.lock().await;
    let node = tree.root_node();

    if let Some(mut child) = node.descendant_for_byte_range(start, end) {
        loop {
            if [*wat::kind::INSTR_PLAIN, *wast::kind::INSTR_PLAIN].contains(&child.kind_id()) {
                let source = document.content.bytes().collect::<Vec<_>>();
                let source = source.as_slice();
                let text = child.utf8_text(source)?;
                contents.push(lsp::MarkedString::String(String::from(text)));
                range = Some(crate::util::node::range(&child));
                break;
            }

            if [*wat::kind::INSTR, *wast::kind::INSTR].contains(&child.kind_id()) {
                let source = document.content.bytes().collect::<Vec<_>>();
                let source = source.as_slice();
                let text = child.utf8_text(source)?;
                contents.push(lsp::MarkedString::String(String::from(text)));
                range = Some(crate::util::node::range(&child));
                break;
            }

            if module_fields.contains(&child.kind_id()) {
                let source = document.content.bytes().collect::<Vec<_>>();
                let source = source.as_slice();
                let text = child.utf8_text(source)?;
                contents.push(lsp::MarkedString::String(String::from(text)));
                range = Some(crate::util::node::range(&child));
                break;
            }

            if let Some(parent) = child.parent() {
                child = parent;
            } else {
                break;
            }
        }
    }

    if contents.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lsp::Hover {
            contents: lsp::HoverContents::Array(contents),
            range,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::core;

    #[test]
    fn character_to_line_offset_ok() {
        let line_text = "text";
        let character = line_text.len();
        let result = crate::util::character::line::offset(line_text, character);
        assert!(result.is_ok())
    }

    #[test]
    fn character_to_line_offset_out_of_bounds() {
        let line_text = "text";
        let character = line_text.len() + 1;
        let result = crate::util::character::line::offset(line_text, character);
        assert!(result.is_err())
    }

    #[test]
    fn line_start_ok() {
        let language_id = "wasm.wast";
        let text = String::from("(module)");
        let result = core::Document::new(language_id, text);
        assert!(result.is_ok());
        if let Ok(ref document) = result {
            let source = document.content.chunks().collect::<String>();
            let source = source.as_str();
            let line_starts = crate::util::line::starts(source).collect::<Vec<_>>();
            let line_index = 1;
            let result = crate::util::line::start(document, &line_starts, line_index);
            assert!(result.is_ok())
        }
    }

    #[test]
    fn line_start_out_of_bounds() {
        let language_id = "wasm.wast";
        let text = String::from("(module)");
        let result = core::Document::new(language_id, text);
        assert!(result.is_ok());
        if let Ok(ref document) = result {
            let source = document.content.chunks().collect::<String>();
            let source = source.as_str();
            let line_starts = crate::util::line::starts(source).collect::<Vec<_>>();
            let line_index = 2;
            let result = crate::util::line::start(document, &line_starts, line_index);
            assert!(result.is_err())
        }
    }
}
