mod take_until;
mod types;

pub(crate) use types::*;

use {
    crate::{
        graph::Subgraph,
        lang::Language,
        lsp::{Client, Message},
    },
    crossbeam_channel::{bounded, Receiver},
    dashmap::DashMap,
    lsp_types::{
        request::GotoImplementationResponse, CallHierarchyOutgoingCall, DocumentSymbolResponse,
        GotoDefinitionResponse, Location, Url,
    },
    rayon::prelude::*,
    serde_json,
    std::{
        borrow::BorrowMut,
        cell::RefCell,
        collections::{BTreeMap, HashMap},
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
    take_until::TakeUntilExt,
    walkdir::WalkDir,
};

pub(crate) struct Analyzer {
    pub lang: Box<dyn Language + Sync + Send>,
    client: Client,
    rsp_receiver: Receiver<Message>,
    pub path_map: RefCell<PathMap>,
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
                path_map: RefCell::new(PathMap::new()),
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
        let mut path_map = self.path_map.borrow_mut();
        let path_map: &mut PathMap = (*path_map).borrow_mut();

        let client = &self.client;
        let rsp_receiver = &self.rsp_receiver;

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
                    .inspect(|e| {
                        count.fetch_add(1, Ordering::SeqCst);
                        path_map.insert(e.path().to_path_buf());
                    })
                    .par_bridge()
                    .for_each(|entry| {
                        let path = entry.path();
                        let id = client.document_symbol(path.to_str().unwrap());

                        map.insert(id, path.to_path_buf());
                    });

                is_finished.store(true, Ordering::SeqCst);
            });

            s.spawn(|_| {
                result = rsp_receiver
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

                        let symbols = match rsp {
                            DocumentSymbolResponse::Flat(v) => {
                                println!("{:#?}", v);
                                vec![]
                            }
                            DocumentSymbolResponse::Nested(v) => v,
                        };

                        let path = map.remove(&id).unwrap().1;

                        FileOutline {
                            id: 0,
                            path,
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
            .iter_mut()
            .for_each(|f| f.id = path_map.get(&f.path).unwrap());

        result
    }

    pub fn outgoing_calls(
        &self,
        outlines: &Vec<FileOutline>,
    ) -> HashMap<SymbolLocation, Vec<CallHierarchyOutgoingCall>> {
        let map = DashMap::new();

        let mut result = HashMap::new();
        let count = AtomicUsize::new(0);
        let is_finished = AtomicBool::new(false);

        let client = &self.client;
        let rsp_receiver = &self.rsp_receiver;

        rayon::scope(|s| {
            s.spawn(|_| {
                outlines.iter().for_each(|outline| {
                    let uri = Url::from_file_path(&outline.path).unwrap();
                    let functions = self.lang.all_functions(outline);

                    count.fetch_add(functions.len(), Ordering::SeqCst);

                    functions.par_iter().for_each(|func| {
                        let id = client.outgoing_calls(&uri, func);

                        map.insert(id, SymbolLocation::new(&uri, &func.selection_range.start));
                    })
                });

                is_finished.store(true, Ordering::SeqCst);
            });

            s.spawn(|_| {
                result = rsp_receiver
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

        let client = &self.client;
        let rsp_receiver = &self.rsp_receiver;

        rayon::scope(|s| {
            s.spawn(|_| {
                outlines.iter().for_each(|outline| {
                    let uri = Url::from_file_path(&outline.path).unwrap();
                    let interfaces = self.lang.all_interfaces(outline);

                    count.fetch_add(interfaces.len(), Ordering::SeqCst);

                    interfaces.par_iter().for_each(|interface| {
                        let id = client.implementations(&uri, interface);

                        map.insert(
                            id,
                            SymbolLocation::new(&uri, &interface.selection_range.start),
                        );
                    });
                });

                is_finished.store(true, Ordering::SeqCst);
            });

            // FIXME: it will be blocked forever if there is no interface
            s.spawn(|_| {
                result = rsp_receiver
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

    pub fn subgraphs(&self, files: &[FileOutline]) -> Vec<Subgraph> {
        let mut dirs = BTreeMap::new();
        for f in files {
            let parent = f.path.parent().unwrap();
            dirs.entry(parent)
                .or_insert(Vec::new())
                .push(f.path.clone());
        }

        fn subgraph_recursive(
            parent: &Path,
            dirs: &BTreeMap<&Path, Vec<PathBuf>>,
            map: &PathMap,
        ) -> Vec<Subgraph> {
            dirs.iter()
                .filter(|(dir, _)| dir.parent().unwrap() == parent)
                .map(|(dir, v)| Subgraph {
                    title: dir.file_name().unwrap().to_str().unwrap().into(),
                    nodes: v
                        .iter()
                        .map(|path| map.get(&path).unwrap().to_string())
                        .collect::<Vec<_>>(),
                    subgraphs: subgraph_recursive(dir, dirs, map),
                })
                .collect::<Vec<_>>()
        }

        subgraph_recursive(dirs.keys().next().unwrap(), &dirs, &self.path_map.borrow())
    }
}

pub struct PathMap {
    // analysis_root: PathBuf,
    // source: HashMap<PathBuf, u32>,
    // dependencies: HashMap<PathBuf, u32>,
    map: HashMap<PathBuf, PathId>,
    next_id: PathId,
}

impl PathMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 1,
        }
    }

    fn insert(&mut self, path: PathBuf) -> PathId {
        match self.map.try_insert(path, self.next_id) {
            Ok(id) => {
                self.next_id += 1;
                id.to_owned()
            }
            Err(e) => e.value,
        }
    }

    pub fn get(&self, path: &Path) -> Option<u32> {
        self.map.get(path).copied()
    }
}
