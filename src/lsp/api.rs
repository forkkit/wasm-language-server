//! Definitions for the request handlers.

use crate::lsp::server::Server;
use jsonrpc_core::Result;
use log;
use lsp_types::*;
use tower_lsp::{Client, LanguageServer};

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    fn initialize(&self, _: &Client, params: InitializeParams) -> Result<InitializeResult> {
        log::info!("{:?}", params);
        let capabilities = crate::lsp::capabilities::capabilities();
        Ok(InitializeResult {
            capabilities,
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, client: &Client, _: InitializedParams) {
        client.log_message(MessageType::Info, "server initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, client: &Client, params: DidOpenTextDocumentParams) {
        // FIXME: remove on next tower-lsp release
        let client: &'static Client = unsafe { std::mem::transmute(client) };
        crate::service::synchronizer::document::open(self.session.clone(), client, params)
            .await
            .unwrap()
    }

    async fn did_change(&self, client: &Client, params: DidChangeTextDocumentParams) {
        // FIXME: remove on next tower-lsp release
        let client: &'static Client = unsafe { std::mem::transmute(client) };
        crate::service::synchronizer::document::change(self.session.clone(), client, params)
            .await
            .unwrap()
    }

    async fn did_close(&self, client: &Client, params: DidCloseTextDocumentParams) {
        // FIXME: remove on next tower-lsp release
        let client: &'static Client = unsafe { std::mem::transmute(client) };
        crate::service::synchronizer::document::close(self.session.clone(), client, params)
            .await
            .unwrap()
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        crate::service::elaborator::document_symbol(self.session.clone(), params).await
    }
}