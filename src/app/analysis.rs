use {
    crate::{
        lang::Language,
        lsp::{Client, Message},
        utils::take_until::TakeUntilExt,
    },
    crossbeam_channel::{bounded, Receiver},
    dashmap::DashMap,
    lsp_types::{
        request::GotoImplementationResponse, CallHierarchyOutgoingCall, DocumentSymbol,
        DocumentSymbolResponse, GotoDefinitionResponse, Location, Position, SymbolKind, Url,
    },
    rayon::prelude::*,
    serde_json,
    std::{
        collections::HashMap,
        fmt::Display,
        hash::Hash,
        io::{self, BufReader},
        path::{Path, PathBuf},
        process::Child,
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc::sync_channel,
        },
        thread::{self, sleep},
        time,
    },
    walkdir::WalkDir,
};

pub(crate) struct Analyzer {
    lang: Box<dyn Language + Sync + Send>,
    client: Client,
    rsp_receiver: Receiver<Message>,
}

impl Analyzer {
    pub fn new(
        lang: Box<dyn Language + Sync + Send>,
        mut lsp_server: Child,
        path: &Path,
    ) -> (Self, thread::JoinHandle<io::Result<()>>) {
        let mut stdin = lsp_server.stdin.take().unwrap();
        let stdout = lsp_server.stdout.take().unwrap();

        let (req_sender, req_reciver) = sync_channel::<Message>(4);
        thread::spawn(move || {
            req_reciver
                .into_iter()
                .try_for_each(|it| it.write(&mut stdin))
        });

        let (rsp_sender, rsp_receiver) = bounded::<Message>(20);
        let io_thread = thread::spawn(move || {
            let mut buf_reader = BufReader::new(stdout);

            while let Some(msg) = Message::read(&mut buf_reader)? {
                let is_exit = match &msg {
                    Message::Notification(n) => n.is_exit(),
                    _ => false,
                };

                rsp_sender.send(msg).unwrap();

                if is_exit {
                    break;
                }
            }

            Ok(())
        });

        let client = Client::new(req_sender);

        client.initialize(path.to_str().unwrap(), &rsp_receiver);

        // wait for cargo check
        let dur = time::Duration::from_millis(1200);
        sleep(dur);

        (
            Self {
                lang,
                client,
                rsp_receiver,
            },
            io_thread,
        )
    }

    pub(crate) fn file_outlines(&self, dir: &Path) -> Vec<FileOutline> {
        let entry = self.lang.entry(dir);

        let map = DashMap::new();
        let count = AtomicUsize::new(0);
        let is_finished = AtomicBool::new(false);

        let mut result = vec![];

        rayon::scope(|s| {
            s.spawn(|_| {
                WalkDir::new(dir)
                    .into_iter()
                    .filter_entry(|e| {
                        let path = e.path();

                        entry.exclude.iter().all(|it| path != it)
                            && entry.include.iter().any(|it| path.starts_with(it))
                    })
                    .filter_map(|e| e.ok().filter(|e| e.file_type().is_file()))
                    .filter(|e| {
                        let path = e.path();

                        let ext = path.extension().unwrap_or_default();
                        entry.extensions.iter().any(|it| it.as_str() == ext)
                    })
                    .inspect(|_| {
                        count.fetch_add(1, Ordering::SeqCst);
                    })
                    .par_bridge()
                    .for_each(|entry| {
                        let path = entry.path();
                        let id = self.client.document_symbol(path.to_str().unwrap());

                        map.insert(id, path.to_path_buf());
                    });

                is_finished.store(true, Ordering::SeqCst);
            });

            s.spawn(|_| {
                result = self
                    .rsp_receiver
                    .iter()
                    .filter_map(|msg| match msg {
                        Message::Notification(_) | Message::Request(_) => None,
                        Message::Response(rsp) => Some(rsp),
                    })
                    .map(|rsp| {
                        let id = rsp.id;

                        let rsp = rsp.result.unwrap();
                        let rsp = serde_json::from_value::<DocumentSymbolResponse>(rsp).unwrap();

                        count.fetch_sub(1, Ordering::SeqCst);

                        let mut symbols = match rsp {
                            DocumentSymbolResponse::Flat(v) => {
                                println!("{:#?}", v);
                                vec![]
                            }
                            DocumentSymbolResponse::Nested(v) => v,
                        };

                        symbols
                            .iter_mut()
                            .for_each(|symbol| self.trim_symbols(symbol));

                        FileOutline {
                            path: map.remove(&id).unwrap().1,
                            symbols,
                        }
                    })
                    .take_until(|_| {
                        is_finished.load(Ordering::SeqCst) && count.load(Ordering::SeqCst) == 0
                    })
                    .collect();
            })
        });

        result
    }

    fn trim_symbols(&self, symbol: &mut DocumentSymbol) {
        match symbol.kind {
            SymbolKind::STRUCT | SymbolKind::ENUM | SymbolKind::CLASS => {
                symbol.children = None;
            }
            _ => {
                symbol.children.as_mut().map(|symbols| {
                    symbols.iter_mut().for_each(|symbol| {
                        self.trim_symbols(symbol);
                    });
                    symbols
                });
            }
        }
    }

    pub fn outgoing_calls(
        &self,
        outlines: &Vec<FileOutline>,
    ) -> HashMap<SymbolLocation, Vec<CallHierarchyOutgoingCall>> {
        let map = DashMap::new();

        let mut result = HashMap::new();
        let count = AtomicUsize::new(0);
        let is_finished = AtomicBool::new(false);

        rayon::scope(|s| {
            s.spawn(|_| {
                outlines.iter().for_each(|outline| {
                    let uri = Url::from_file_path(&outline.path).unwrap();
                    let functions = self.lang.all_functions(outline);

                    count.fetch_add(functions.len(), Ordering::SeqCst);

                    functions.par_iter().for_each(|func| {
                        let id = self.client.outgoing_calls(&uri, func);

                        map.insert(id, SymbolLocation::new(&uri, &func.selection_range.start));
                    })
                });

                is_finished.store(true, Ordering::SeqCst);
            });

            s.spawn(|_| {
                result = self
                    .rsp_receiver
                    .iter()
                    .filter_map(|msg| match msg {
                        Message::Notification(_) | Message::Request(_) => None,
                        Message::Response(rsp) => Some(rsp),
                    })
                    .inspect(|_| {
                        count.fetch_sub(1, Ordering::SeqCst);
                    })
                    .take_until(|_| {
                        is_finished.load(Ordering::SeqCst) && count.load(Ordering::SeqCst) == 0
                    })
                    .filter_map(|rsp| {
                        let id = rsp.id;

                        // TODO: handling error
                        if let Some(e) = rsp.error {
                            eprint!("{:#?}", e);
                            return None;
                        }

                        let Some(rsp) = rsp.result else {
                            return None;
                        };

                        let locations =
                            serde_json::from_value::<Vec<CallHierarchyOutgoingCall>>(rsp).unwrap();

                        Some((map.remove(&id).unwrap().1, locations))
                    })
                    .collect();
            });
        });

        result
    }

    pub fn interface_implementations(
        &self,
        outlines: &Vec<FileOutline>,
    ) -> HashMap<SymbolLocation, Vec<SymbolLocation>> {
        let map = DashMap::new();
        let count = AtomicUsize::new(0);
        let is_finished = AtomicBool::new(false);

        let mut result = HashMap::new();

        rayon::scope(|s| {
            s.spawn(|_| {
                outlines.iter().for_each(|outline| {
                    let uri = Url::from_file_path(&outline.path).unwrap();
                    let interfaces = self.lang.all_interfaces(outline);

                    count.fetch_add(interfaces.len(), Ordering::SeqCst);

                    interfaces.par_iter().for_each(|interface| {
                        let id = self.client.implementations(&uri, interface);

                        map.insert(
                            id,
                            SymbolLocation::new(&uri, &interface.selection_range.start),
                        );
                    });
                });

                is_finished.store(true, Ordering::SeqCst);
            });

            s.spawn(|_| {
                result = self
                    .rsp_receiver
                    .iter()
                    .filter_map(|msg| match msg {
                        Message::Notification(_) | Message::Request(_) => None,
                        Message::Response(rsp) => Some(rsp),
                    })
                    .inspect(|_| {
                        count.fetch_sub(1, Ordering::SeqCst);
                    })
                    .take_until(|_| {
                        is_finished.load(Ordering::SeqCst) && count.load(Ordering::SeqCst) == 0
                    })
                    .filter_map(|rsp| {
                        let id = rsp.id;

                        // TODO: handling error
                        if let Some(e) = rsp.error {
                            eprint!("{:#?}", e);
                            return None;
                        }

                        let Some(rsp) = rsp.result else {
                            return None;
                        };
                        let rsp =
                            serde_json::from_value::<GotoImplementationResponse>(rsp).unwrap();

                        let locations = match rsp {
                            GotoDefinitionResponse::Scalar(l) => vec![l],
                            GotoDefinitionResponse::Array(ls) => ls,
                            GotoDefinitionResponse::Link(links) => links
                                .into_iter()
                                .map(|link| Location {
                                    uri: link.target_uri,
                                    range: link.target_selection_range,
                                })
                                .collect(),
                        };

                        Some((
                            map.remove(&id).unwrap().1,
                            locations
                                .iter()
                                .map(|location| {
                                    SymbolLocation::new(&location.uri, &location.range.start)
                                })
                                .collect(),
                        ))
                    })
                    .collect();
            })
        });

        result
    }
}

pub(crate) struct FileOutline {
    pub path: PathBuf,
    pub symbols: Vec<DocumentSymbol>,
}

pub type Relations = HashMap<SymbolLocation, Vec<(SymbolLocation, Option<String>)>>;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct SymbolLocation {
    pub path: String,
    pub line: u32,
    pub character: u32,
}

impl SymbolLocation {
    pub fn new(uri: &Url, position: &Position) -> Self {
        Self {
            path: uri.path().to_string().trim_end_matches('/').to_string(),
            line: position.line,
            character: position.character,
        }
    }
}

impl Display for SymbolLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#""{}":"{}_{}""#, self.path, self.line, self.character)
    }
}
