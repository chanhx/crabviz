//! Some Language Server Protocol types used in crabviz, copied from gluon-lang/lsp-types with some modifications.

use {
    serde::{Deserialize, Serialize},
    serde_json::Value,
    serde_repr::*,
};

/// A symbol kind.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
// #[serde(transparent)]
#[repr(u8)]
pub enum SymbolKind {
    File = 1,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
// #[serde(transparent)]
#[repr(u8)]
pub enum SymbolTag {
    /**
     * Render a symbol as obsolete, usually using a strike-out.
     */
    Deprecated = 1,
}

/// Position in a text document expressed as zero-based line and character offset.
/// A position is between two characters like an 'insert' cursor in a editor.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: u32,
    /// Character offset on a line in a document (zero-based). The meaning of this
    /// offset is determined by the negotiated `PositionEncodingKind`.
    ///
    /// If the character value is greater than the line length it defaults back
    /// to the line length.
    pub character: u32,
}

/// A range in a text document expressed as (zero-based) start and end positions.
/// A range is comparable to a selection in an editor. Therefore the end position is exclusive.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Range {
    /// The range's start position.
    pub start: Position,
    /// The range's end position.
    pub end: Position,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbol {
    /// The name of this symbol.
    pub name: String,
    /// More detail for this symbol, e.g the signature of a function. If not provided the
    /// name is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The kind of this symbol.
    pub kind: SymbolKind,
    /// Tags for this completion item.
    ///  since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<SymbolTag>>,
    /// Indicates if this symbol is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use tags instead")]
    pub deprecated: Option<bool>,
    /// The range enclosing this symbol not including leading/trailing whitespace but everything else
    /// like comments. This information is typically used to determine if the the clients cursor is
    /// inside the symbol to reveal in the symbol in the UI.
    pub range: Range,
    /// The range that should be selected and revealed when this symbol is being picked, e.g the name of a function.
    /// Must be contained by the the `range`.
    pub selection_range: Range,
    /// Children of this symbol, e.g. properties of a class.
    #[serde(default = "Vec::default")]
    pub children: Vec<DocumentSymbol>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallHierarchyItem {
    /// The name of this item.
    pub name: String,

    /// The kind of this item.
    pub kind: SymbolKind,

    /// Tags for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<SymbolTag>>,

    /// More detail for this item, e.g. the signature of a function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// The resource identifier of this item.
    pub uri: Url,

    /// The range enclosing this symbol not including leading/trailing whitespace but everything else, e.g. comments and code.
    pub range: Range,

    /// The range that should be selected and revealed when this symbol is being picked, e.g. the name of a function.
    /// Must be contained by the [`range`](#CallHierarchyItem.range).
    pub selection_range: Range,

    /// A data entry field that is preserved between a call hierarchy prepare and incloming calls or outgoing calls requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Represents an incoming call, e.g. a caller of a method or constructor.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallHierarchyIncomingCall {
    /// The item that makes the call.
    pub from: CallHierarchyItem,

    /// The range at which at which the calls appears. This is relative to the caller
    /// denoted by [`this.from`](#CallHierarchyIncomingCall.from).
    pub from_ranges: Vec<Range>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
#[serde(untagged)]
pub enum Url {
    VscodeRepr(VscodeUrl),
}

impl Url {
    pub fn path(&self) -> &str {
        match self {
            Self::VscodeRepr(url) => &url.path,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct VscodeUrl {
    pub scheme: String,
    pub authority: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct Location {
    pub uri: Url,
    pub range: Range,
}

/// Represents a link between a source and a target location.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationLink {
    /// Span of the origin of this link.
    ///
    ///  Used as the underlined span for mouse interaction. Defaults to the word range at
    ///  the mouse position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_selection_range: Option<Range>,

    /// The target resource identifier of this link.
    pub target_uri: Url,

    /// The full target range of this link.
    pub target_range: Range,

    /// The span of this link.
    pub target_selection_range: Range,
}
