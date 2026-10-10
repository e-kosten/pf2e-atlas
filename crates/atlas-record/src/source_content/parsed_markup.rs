//! Private transient nodes for one authored field. Never serialized or stored.
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct ParsedMarkup {
    pub nodes: Vec<ParsedNode>,
}

impl ParsedMarkup {
    pub(super) fn new(nodes: Vec<ParsedNode>) -> Self {
        Self { nodes }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ParsedNode {
    Text {
        text: String,
    },
    HtmlElement {
        tag: String,
        attributes: BTreeMap<String, Option<String>>,
        children: Vec<ParsedNode>,
    },
    ParsedLink {
        link: ParsedLink,
    },
    Foundry {
        node: ParsedMacro,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedLink {
    pub target: ParsedLinkTarget,
    pub label: Option<Vec<ParsedNode>>,
    pub source: ParsedLinkSource,
    pub behavior: ParsedLinkBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ParsedLinkTarget {
    Unresolved {
        target: String,
        fallback_label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedLinkSource {
    pub macro_kind: ParsedLinkMacroKind,
    pub authored_target: String,
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParsedLinkMacroKind {
    Uuid,
    Compendium,
    Embed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ParsedLinkBehavior {
    Reference,
    Embed {
        inline: bool,
        hr: Option<bool>,
        options: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ParsedMacro {
    Check {
        statistic: Option<String>,
        options: BTreeMap<String, String>,
        label: Option<Vec<ParsedNode>>,
    },
    Damage {
        formula: String,
        options: BTreeMap<String, String>,
        damage_parts: Vec<ParsedDamagePart>,
        label: Option<Vec<ParsedNode>>,
    },
    InlineCommand {
        command: String,
        arguments: String,
        options: BTreeMap<String, String>,
        label: Option<Vec<ParsedNode>>,
    },
    Template {
        shape: Option<String>,
        options: BTreeMap<String, String>,
        label: Option<Vec<ParsedNode>>,
    },
    Localize {
        key: String,
        label: Option<Vec<ParsedNode>>,
        resolved: Option<Vec<ParsedNode>>,
    },
    UnknownFoundry {
        name: String,
        body: Option<String>,
        label: Option<Vec<ParsedNode>>,
        raw: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedDamagePart {
    pub formula: String,
    pub damage_type: Option<String>,
}
