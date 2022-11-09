use {
    super::message::{Message, Notification, Request, RequestId},
    crossbeam_channel::Receiver,
    lsp_types::{
        notification::Notification as _,
        request::{
            CallHierarchyOutgoingCalls, DocumentSymbolRequest, GotoImplementation, Initialize,
        },
        ClientCapabilities, ClientInfo, DocumentSymbol, DocumentSymbolClientCapabilities,
        InitializeParams, InitializeResult, InitializedParams, TextDocumentClientCapabilities,
        TextDocumentIdentifier, Url,
    },
    std::{
        path::Path,
        process,
        sync::{
            atomic::{AtomicI32, Ordering},
            mpsc::SyncSender,
        },
    },
};

pub struct Client {
    request_id: AtomicI32,
    req_sender: SyncSender<Message>,
}

impl Client {
    pub(crate) fn new(req_sender: SyncSender<Message>) -> Self {
        Self {
            request_id: AtomicI32::new(0),
            req_sender,
        }
    }

    pub(crate) fn initialize(&self, workspace: &str, rsp_receiver: &Receiver<Message>) {
        let workspace = shellexpand::full(workspace)
            .map(|path| std::path::Path::new(path.as_ref()).canonicalize().unwrap())
            .unwrap();

        self.initialize_start(&workspace);

        let msg = rsp_receiver.iter().next();

        match msg {
            Some(Message::Response(rsp)) => {
                let rsp = rsp.result.unwrap();
                let result = serde_json::from_value::<InitializeResult>(rsp).unwrap();

                self.initialize_finish(result);
            }
            _ => unimplemented!(),
        }
    }

    fn initialize_start(&self, workspace: &Path) {
        let root_uri = Some(Url::from_directory_path(workspace).unwrap());

        let params = InitializeParams {
            process_id: Some(process::id()),
            root_uri,
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    document_symbol: Some(DocumentSymbolClientCapabilities {
                        hierarchical_document_symbol_support: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            client_info: Some(ClientInfo {
                name: "crabviz".into(),
                version: None,
            }),
            ..Default::default()
        };

        self.send_request::<Initialize>(params);
    }

    fn initialize_finish(&self, result: InitializeResult) {
        let initialized_params = InitializedParams {};
        let notification = Message::Notification(Notification::new(
            lsp_types::notification::Initialized::METHOD.to_string(),
            initialized_params,
        ));

        self.req_sender.send(notification).unwrap();
    }

    pub(crate) fn document_symbol(&self, path: &str) -> RequestId {
        let path = shellexpand::full(path)
            .map(|path| std::path::Path::new(path.as_ref()).canonicalize().unwrap())
            .unwrap();

        let params = lsp_types::DocumentSymbolParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: Url::from_directory_path(path).unwrap(),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request::<DocumentSymbolRequest>(params)
    }

    pub(crate) fn outgoing_calls(&self, uri: &Url, func: &DocumentSymbol) -> RequestId {
        let params = lsp_types::CallHierarchyOutgoingCallsParams {
            item: lsp_types::CallHierarchyItem {
                name: func.name.clone(),
                kind: func.kind,
                tags: None,
                detail: None,
                uri: uri.clone(),
                range: func.range,
                selection_range: func.selection_range,
                data: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request::<CallHierarchyOutgoingCalls>(params)
    }

    pub(crate) fn implementations(&self, uri: &Url, interface: &DocumentSymbol) -> RequestId {
        let params = lsp_types::request::GotoImplementationParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: interface.selection_range.start,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request::<GotoImplementation>(params)
    }

    fn send_request<R>(&self, params: R::Params) -> RequestId
    where
        R: lsp_types::request::Request,
    {
        let id: RequestId = self.request_id.fetch_add(1, Ordering::SeqCst).into();
        let msg = Message::Request(Request::new(id.clone(), R::METHOD.into(), params));

        self.req_sender.send(msg).unwrap();

        id
    }
}
