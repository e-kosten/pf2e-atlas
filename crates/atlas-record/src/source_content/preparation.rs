use scraper::{Html, Selector};

use crate::{FoundryLink, FoundryLinkBehavior, FoundryNode, RichNode};

use super::{
    CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentDiagnosticCode, ContentInteraction,
    ContentInteractionKind, ContentInterpretationDiagnostic, ContentReferenceKind,
    ContentReferenceOccurrence, ContentReferenceResolution, ContentReferenceResolver,
    ContentReferenceTarget, ContentVisibilityRule, LocalizationResolver, PreparedSourceContent,
    SourceContentLocator, parse_foundry_content_with_localization,
};

/// Interpret an authored HTML field once for all projections. Callers must use
/// upstream format metadata when choosing fields (for example HTML journal
/// pages). Markdown/plain-text document formats are not inferred from strings.
/// Context inputs are explicit; no database,
/// UI routes, document preparation, formula evaluation or embedding is involved.
pub fn prepare_source_content(
    locator: SourceContentLocator,
    markup: &str,
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
    localization: Option<&dyn LocalizationResolver>,
    references: Option<&dyn ContentReferenceResolver>,
) -> Result<PreparedSourceContent, html2text::Error> {
    let parsed = parse_foundry_content_with_localization(markup, localization);
    let mut projection = Projection {
        locator: &locator,
        audience,
        resolver: references,
        references: Vec::new(),
        interactions: Vec::new(),
        diagnostics: Vec::new(),
    };
    let audiences = match visibility {
        ContentVisibilityRule::All => Vec::new(),
        ContentVisibilityRule::Gm => vec!["gm".into()],
        ContentVisibilityRule::Owner => vec!["owner".into()],
        ContentVisibilityRule::None => vec!["none".into()],
    };
    let html = projection.nodes(
        &parsed.document.nodes,
        "",
        &audiences,
        audience.permits(visibility),
        false,
    );
    let html = sanitizer().clean(&html).to_string();
    let text = super::text_projection::plain_text(&html)?;
    Ok(PreparedSourceContent {
        locator: locator.clone(),
        interpretation_version: CONTENT_INTERPRETATION_VERSION.to_string(),
        html,
        text,
        references: projection.references,
        interactions: projection.interactions,
        diagnostics: projection.diagnostics,
    })
}

fn sanitizer() -> ammonia::Builder<'static> {
    let mut builder = ammonia::Builder::default();
    builder
        .add_generic_attributes(["data-atlas-reference", "data-atlas-interaction"])
        .add_tag_attributes("ol", ["start"])
        .add_tag_attributes("li", ["value"])
        .add_tag_attributes("td", ["colspan", "rowspan"])
        .add_tag_attributes("th", ["colspan", "rowspan"]);
    builder
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

struct Projection<'a> {
    locator: &'a SourceContentLocator,
    audience: ContentAudience,
    resolver: Option<&'a dyn ContentReferenceResolver>,
    references: Vec<ContentReferenceOccurrence>,
    interactions: Vec<ContentInteraction>,
    diagnostics: Vec<ContentInterpretationDiagnostic>,
}

impl Projection<'_> {
    fn issue(&mut self, path: &str, code: ContentDiagnosticCode, detail: &str) {
        self.diagnostics.push(ContentInterpretationDiagnostic {
            path: path.to_string(),
            code,
            detail: detail.to_string(),
        });
    }

    fn nodes(
        &mut self,
        nodes: &[RichNode],
        path: &str,
        audiences: &[String],
        visible: bool,
        in_link: bool,
    ) -> String {
        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                self.node(
                    node,
                    &format!("{path}/{index}"),
                    audiences,
                    visible,
                    in_link,
                )
            })
            .collect()
    }

    fn node(
        &mut self,
        node: &RichNode,
        path: &str,
        audiences: &[String],
        visible: bool,
        in_link: bool,
    ) -> String {
        match node {
            RichNode::Text { text } => {
                self.literal_diagnostics(path, text);
                if visible { escape(text) } else { String::new() }
            }
            RichNode::HtmlElement {
                tag,
                attributes,
                children,
            } => {
                let mut audiences = audiences.to_vec();
                let visibility = attributes.get("data-visibility").and_then(Option::as_deref);
                if let Some(visibility) = visibility {
                    audiences.push(visibility.to_string());
                }
                let visible = visible && visibility.is_none_or(|value| self.permits(path, value));
                let excluded = matches!(tag.as_str(), "script" | "style" | "iframe" | "object");
                if excluded {
                    self.issue(path, ContentDiagnosticCode::ExcludedHtmlContent, tag);
                }
                let visible = visible && !excluded;
                let mut attributes = attributes.clone();
                // Only this preparation call may create its fact markers.
                attributes.retain(|name, _| !name.starts_with("data-atlas-"));
                if tag == "a" {
                    let uuid = attributes.get("data-uuid").and_then(Option::as_deref);
                    let has_uuid = uuid.is_some();
                    let href = attributes.get("href").and_then(Option::as_deref);
                    if let Some(target) = uuid.or(href).map(str::to_string) {
                        let kind = if has_uuid {
                            ContentReferenceKind::Uuid
                        } else {
                            ContentReferenceKind::HtmlLink
                        };
                        let (ordinal, _, _) =
                            self.reference(path, &audiences, visible, kind, &target);
                        attributes.insert("data-atlas-reference".into(), Some(ordinal.to_string()));
                        if has_uuid {
                            // Product routes bind to the sidecar target at rendering.
                            attributes.remove("href");
                        }
                    }
                }
                let children_html =
                    self.nodes(children, path, &audiences, visible, in_link || tag == "a");
                if !visible {
                    return String::new();
                }
                let glyph = tag == "pf2-action"
                    || attributes
                        .get("class")
                        .and_then(Option::as_deref)
                        .is_some_and(|classes| {
                            classes
                                .split_whitespace()
                                .any(|class| matches!(class, "action-glyph" | "pf2-icon"))
                        });
                if glyph {
                    if contains_interpretation(children) {
                        self.issue(
                            path,
                            ContentDiagnosticCode::UnknownActionGlyph,
                            "Glyph element contains enrichments; retain its interpreted children",
                        );
                        return format!("<span>{children_html}</span>");
                    }
                    let value = attributes
                        .get("glyph")
                        .or_else(|| attributes.get("action"))
                        .and_then(Option::as_deref)
                        .map(str::to_string)
                        .unwrap_or_else(|| simple_text(children));
                    return format!("<span>{}</span>", escape(&self.action(path, &value)));
                }
                let attributes = attributes
                    .iter()
                    .map(|(name, value)| {
                        format!(
                            " {name}=\"{}\"",
                            escape(value.as_deref().unwrap_or_default())
                        )
                    })
                    .collect::<String>();
                // Nested authored anchors retain occurrence facts but are not
                // emitted as invalid nested links whose label boundaries shift.
                let tag = if tag == "a" && in_link { "span" } else { tag };
                if matches!(
                    tag,
                    "area"
                        | "base"
                        | "br"
                        | "col"
                        | "embed"
                        | "hr"
                        | "img"
                        | "input"
                        | "link"
                        | "meta"
                        | "param"
                        | "source"
                        | "track"
                        | "wbr"
                ) {
                    format!("<{tag}{attributes}>")
                } else {
                    format!("<{tag}{attributes}>{children_html}</{tag}>")
                }
            }
            RichNode::FoundryLink { link } => self.link(link, path, audiences, visible, in_link),
            RichNode::Foundry { node } => self.foundry(node, path, audiences, visible, in_link),
        }
    }

    fn permits(&mut self, path: &str, value: &str) -> bool {
        let rule = match value {
            "all" => ContentVisibilityRule::All,
            "gm" => ContentVisibilityRule::Gm,
            "owner" => ContentVisibilityRule::Owner,
            "none" => ContentVisibilityRule::None,
            _ => {
                self.issue(path, ContentDiagnosticCode::UnknownVisibility, value);
                return false;
            }
        };
        self.audience.permits(rule)
    }

    fn literal_diagnostics(&mut self, path: &str, text: &str) {
        // Signals left as text by the shared parser are retained, not silently
        // treated as interpreted controls. This reports syntax, not field loss.
        let unparsed_macro = text.match_indices('@').any(|(index, _)| {
            let suffix = &text[index + 1..];
            suffix.split_once('[').is_some_and(|(name, _)| {
                !name.is_empty()
                    && name
                        .chars()
                        .all(|character| character.is_ascii_alphabetic())
            })
        });
        if unparsed_macro || text.contains("[[/") {
            self.issue(
                path,
                ContentDiagnosticCode::MalformedSyntax,
                "Unparsed enrichment syntax retained as literal text",
            );
        }
    }

    fn reference(
        &mut self,
        path: &str,
        audiences: &[String],
        visible: bool,
        kind: ContentReferenceKind,
        target: &str,
    ) -> (usize, Option<String>, ContentReferenceResolution) {
        let resolved = if matches!(kind, ContentReferenceKind::HtmlLink) {
            None
        } else {
            self.resolver
                .and_then(|resolver| resolver.resolve_reference(self.locator, target))
        };
        let display = resolved
            .as_ref()
            .and_then(|resolved| resolved.display_name.clone());
        let resolution = if let Some(resolved) = resolved {
            if let ContentReferenceTarget::Url { url } = &resolved.target {
                if safe_url(url) {
                    ContentReferenceResolution::Resolved(resolved.target)
                } else {
                    ContentReferenceResolution::Blocked
                }
            } else {
                ContentReferenceResolution::Resolved(resolved.target)
            }
        } else if matches!(kind, ContentReferenceKind::HtmlLink) {
            if safe_url(target) {
                ContentReferenceResolution::UnverifiedUrl {
                    url: target.to_string(),
                }
            } else {
                ContentReferenceResolution::Blocked
            }
        } else {
            ContentReferenceResolution::Unresolved
        };
        match &resolution {
            ContentReferenceResolution::Unresolved => {
                self.issue(path, ContentDiagnosticCode::UnresolvedReference, target)
            }
            ContentReferenceResolution::Blocked => {
                self.issue(path, ContentDiagnosticCode::BlockedDestination, target)
            }
            _ => {}
        }
        let ordinal = self.references.len();
        self.references.push(ContentReferenceOccurrence {
            ordinal,
            path: path.to_string(),
            kind,
            authored_target: target.to_string(),
            resolution: resolution.clone(),
            visible,
            audiences: audiences.to_vec(),
        });
        (ordinal, display, resolution)
    }

    fn link(
        &mut self,
        link: &FoundryLink,
        path: &str,
        audiences: &[String],
        visible: bool,
        in_link: bool,
    ) -> String {
        let kind = match &link.behavior {
            FoundryLinkBehavior::Embed { options, .. } => {
                self.issue(
                    path,
                    ContentDiagnosticCode::EmbedNotExpanded,
                    &link.source.authored_target,
                );
                ContentReferenceKind::Embed {
                    options: options.clone(),
                }
            }
            FoundryLinkBehavior::Reference => match link.source.macro_kind {
                crate::FoundryLinkMacroKind::Uuid => ContentReferenceKind::Uuid,
                _ => ContentReferenceKind::Compendium,
            },
        };
        let (ordinal, display, resolution) =
            self.reference(path, audiences, visible, kind, &link.source.authored_target);
        let label = link
            .label
            .as_deref()
            .map(|nodes| self.nodes(nodes, &format!("{path}/label"), audiences, visible, true))
            .unwrap_or_else(|| escape(display.as_deref().unwrap_or(&link.source.authored_target)));
        if !visible {
            return String::new();
        }
        // Identity markers do not hard-code CLI/web route syntax.
        let href = match resolution {
            ContentReferenceResolution::Resolved(ContentReferenceTarget::Url { url })
                if !in_link =>
            {
                format!(" href=\"{}\"", escape(&url))
            }
            _ => String::new(),
        };
        let tag = if href.is_empty() { "span" } else { "a" };
        format!("<{tag}{href} data-atlas-reference=\"{ordinal}\">{label}</{tag}>")
    }

    fn foundry(
        &mut self,
        node: &FoundryNode,
        path: &str,
        audiences: &[String],
        visible: bool,
        in_link: bool,
    ) -> String {
        if let FoundryNode::Localize {
            key,
            resolved,
            label,
        } = node
        {
            if let Some(label) = label {
                self.issue(path, ContentDiagnosticCode::IgnoredLocalizationLabel, key);
                // Retain authored label references, without making an ignored
                // label visible or substituting it for the localized body.
                self.nodes(label, &format!("{path}/label"), audiences, false, in_link);
            }
            return if let Some(nodes) = resolved {
                self.nodes(
                    nodes,
                    &format!("{path}/localize"),
                    audiences,
                    visible,
                    in_link,
                )
            } else {
                self.issue(path, ContentDiagnosticCode::UnresolvedLocalization, key);
                if visible {
                    escape(&format!("@Localize[{key}]"))
                } else {
                    String::new()
                }
            };
        }
        let label = match node {
            FoundryNode::Check { label, .. }
            | FoundryNode::Damage { label, .. }
            | FoundryNode::InlineCommand { label, .. }
            | FoundryNode::Template { label, .. }
            | FoundryNode::Trait { label, .. }
            | FoundryNode::UnknownFoundry { label, .. } => label.as_deref(),
            _ => None,
        };
        let unknown = matches!(node, FoundryNode::UnknownFoundry { .. });
        let label_html = label.map(|nodes| {
            self.nodes(
                nodes,
                &format!("{path}/label"),
                audiences,
                visible && !unknown,
                in_link,
            )
        });
        let mut interaction = None;
        let text = match node {
            FoundryNode::Check {
                statistic, options, ..
            } => {
                let mut options = options.clone();
                let show_dc = options
                    .get("showDC")
                    .map(|value| self.permits(path, value))
                    .unwrap_or_else(|| self.audience.permits(self.audience.implicit_check_dc));
                if !show_dc {
                    options.remove("dc");
                }
                let statistic = statistic.clone().or_else(|| options.get("type").cloned());
                if statistic.is_none() {
                    self.issue(path, ContentDiagnosticCode::MissingCheckType, "@Check");
                }
                let mut text = statistic
                    .clone()
                    .unwrap_or_else(|| "[missing check type]".into());
                if options.contains_key("basic") {
                    text = format!("basic {text}");
                }
                if let Some(dc) = options.get("dc") {
                    text = format!("DC {dc} {text}");
                }
                if let Some(against) = options.get("against").or_else(|| options.get("defense")) {
                    text.push_str(&format!(" vs {against} DC"));
                }
                self.dynamic(path, options.values().map(String::as_str));
                interaction = Some(ContentInteractionKind::Check { statistic, options });
                text
            }
            FoundryNode::Damage {
                formula, options, ..
            } => {
                self.dynamic(path, std::iter::once(formula.as_str()));
                interaction = Some(ContentInteractionKind::Damage {
                    formula: formula.clone(),
                    options: options.clone(),
                });
                formula.clone()
            }
            FoundryNode::InlineCommand {
                command,
                arguments,
                options,
                ..
            } => {
                self.dynamic(path, std::iter::once(arguments.as_str()));
                interaction = Some(ContentInteractionKind::Command {
                    command: command.clone(),
                    arguments: arguments.clone(),
                    options: options.clone(),
                });
                format!("/{command} {arguments}")
            }
            FoundryNode::Template { shape, options, .. } => {
                let distance = options.get("distance").map(String::as_str).unwrap_or("");
                let valid = shape.as_deref().is_some_and(|shape| {
                    matches!(
                        shape,
                        "burst" | "cone" | "cube" | "cylinder" | "emanation" | "line" | "square"
                    )
                }) && distance.parse::<f64>().is_ok_and(f64::is_finite)
                    && options
                        .get("width")
                        .is_none_or(|width| width.parse::<f64>().is_ok_and(f64::is_finite));
                let text = if valid {
                    let mut text = format!("{distance}-foot {}", shape.as_deref().unwrap_or(""));
                    if let Some(width) = options.get("width") {
                        text.push_str(&format!(" (width {width} feet)"));
                    }
                    text
                } else {
                    self.issue(
                        path,
                        ContentDiagnosticCode::InvalidTemplate,
                        "Authored shape/distance/width retained; no runtime area inferred",
                    );
                    format!(
                        "[template: {}; distance={distance}]",
                        shape.as_deref().unwrap_or("")
                    )
                };
                self.dynamic(path, options.values().map(String::as_str));
                interaction = Some(ContentInteractionKind::Template {
                    shape: shape.clone(),
                    options: options.clone(),
                });
                text
            }
            FoundryNode::ActionGlyph { action } => self.action(path, action),
            FoundryNode::Trait { traits, .. } => traits.join(", "),
            FoundryNode::UnknownFoundry { raw, .. } => {
                self.issue(path, ContentDiagnosticCode::UnknownMacro, raw);
                raw.clone()
            }
            FoundryNode::Localize { .. } => String::new(),
        };
        if !visible {
            return String::new();
        }
        let marker = if let Some(kind) = interaction {
            let ordinal = self.interactions.len();
            self.interactions.push(ContentInteraction {
                ordinal,
                path: path.to_string(),
                kind,
            });
            format!(" data-atlas-interaction=\"{ordinal}\"")
        } else {
            String::new()
        };
        // Authored labels remain labels. Mechanics live in the interaction
        // sidecar, rather than silently adding parentheses to custom prose.
        let body = if unknown {
            escape(&text)
        } else {
            label_html.unwrap_or_else(|| escape(&text))
        };
        format!("<span{marker}>{body}</span>")
    }

    fn dynamic<'a>(&mut self, path: &str, values: impl Iterator<Item = &'a str>) {
        if values
            .into_iter()
            .any(|value| value.contains('@') || value.contains("resolve("))
        {
            self.issue(
                path,
                ContentDiagnosticCode::RuntimeContextRequired,
                "Authored expression retained without evaluation",
            );
        }
    }

    fn action(&mut self, path: &str, value: &str) -> String {
        match value.trim() {
            "F" | "free" | "0" => "free action".into(),
            "R" | "reaction" => "reaction".into(),
            "1" => "one action".into(),
            "2" => "two actions".into(),
            "3" => "three actions".into(),
            "1/2" => "one or two actions".into(),
            "1 - 3" | "1 – 3" => "one to three actions".into(),
            "2/3" => "two or three actions".into(),
            "3,3" => "two rounds".into(),
            _ => {
                self.issue(path, ContentDiagnosticCode::UnknownActionGlyph, value);
                format!("[action glyph: {value}]")
            }
        }
    }
}

// Let the same URL sanitizer decide both navigation and occurrence status.
fn safe_url(url: &str) -> bool {
    let html = sanitizer()
        .clean(&format!("<a href=\"{}\">link</a>", escape(url)))
        .to_string();
    let fragment = Html::parse_fragment(&html);
    Selector::parse("a")
        .ok()
        .and_then(|selector| {
            fragment
                .select(&selector)
                .next()
                .map(|element| element.value().attr("href").is_some())
        })
        .unwrap_or(false)
}

fn simple_text(nodes: &[RichNode]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            RichNode::Text { text } => text.clone(),
            RichNode::HtmlElement { children, .. } => simple_text(children),
            _ => String::new(),
        })
        .collect()
}

fn contains_interpretation(nodes: &[RichNode]) -> bool {
    nodes.iter().any(|node| match node {
        RichNode::Text { .. } => false,
        RichNode::HtmlElement {
            tag,
            attributes,
            children,
        } => {
            (tag == "a"
                && (attributes.contains_key("href") || attributes.contains_key("data-uuid")))
                || contains_interpretation(children)
        }
        RichNode::FoundryLink { .. } | RichNode::Foundry { .. } => true,
    })
}
