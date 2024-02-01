use {
    super::Language,
    crate::lsp_types::{DocumentSymbol, SymbolKind},
};

pub(crate) struct Rust;

impl Language for Rust {
    fn filter_symbol(&self, symbol: &DocumentSymbol) -> bool {
        match symbol.kind {
            SymbolKind::Constant | SymbolKind::Field | SymbolKind::EnumMember => false,
            // any better wasys?
            SymbolKind::Module if symbol.name == "tests" => false,
            _ => true,
        }
    }
}
