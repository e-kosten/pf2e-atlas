//! Bounded, nonpersisted selected-spell damage display. This is not the
//! heightening evaluator and does not participate in default content rendering.
use super::{RecordSurfaceContentContext, RecordSurfaceContentIssueKind, render_nodes_plain_text};
use crate::FoundryNode;

pub(super) fn selected_damage_text(
    node: &FoundryNode,
    context: &RecordSurfaceContentContext,
) -> Result<Option<String>, RecordSurfaceContentIssueKind> {
    let FoundryNode::Damage {
        formula,
        options,
        label,
        ..
    } = node
    else {
        return Ok(None);
    };
    if label
        .as_deref()
        .map(render_nodes_plain_text)
        .is_some_and(|text| !text.trim().is_empty())
    {
        return Ok(None);
    }
    // The aggregate is canonical typed evidence. Fragmented damage_parts are
    // deliberately neither read nor joined by this selected-content policy.
    if !formula.contains('@') && !formula.contains("ternary") {
        return Ok(None);
    }
    let unsupported = RecordSurfaceContentIssueKind::UnsupportedDamage;
    if !options.is_empty() {
        return Err(unsupported);
    }
    let dice = Dice::parse(formula).ok_or(unsupported)?;
    let RecordSurfaceContentContext::Spell { cast_rank, .. } = context else {
        return Err(RecordSurfaceContentIssueKind::MissingContext);
    };
    if !(1..=10).contains(cast_rank) {
        return Err(unsupported);
    }
    let count = dice.count.evaluate(*cast_rank);
    count.checked_mul(dice.sides).ok_or(unsupported)?;
    Ok(Some(format!(
        "{count}d{} persistent {}",
        dice.sides, dice.damage_type
    )))
}

#[derive(Debug)]
struct Dice {
    count: Count,
    sides: u16,
    damage_type: String,
}
#[derive(Debug)]
enum Count {
    Constant(u16),
    Threshold {
        rank: u8,
        count: u16,
        otherwise: Box<Count>,
    },
}
impl Count {
    fn evaluate(&self, rank: u8) -> u16 {
        match self {
            Self::Constant(count) => *count,
            Self::Threshold {
                rank: threshold,
                count,
                otherwise,
            } => {
                if rank >= *threshold {
                    *count
                } else {
                    otherwise.evaluate(rank)
                }
            }
        }
    }
}
impl Dice {
    fn parse(formula: &str) -> Option<Self> {
        let mut parser = Parser { remaining: formula };
        parser.take("(")?;
        let count = parser.count(11, 0)?;
        parser.take(")d")?;
        let sides = parser.positive()?;
        parser.take("[persistent,")?;
        let damage_type = parser.remaining.strip_suffix(']')?;
        if damage_type.is_empty()
            || !damage_type.split('-').all(|token| {
                !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_lowercase())
            })
        {
            return None;
        }
        Some(Self {
            count,
            sides,
            damage_type: damage_type.to_string(),
        })
    }
}
struct Parser<'a> {
    remaining: &'a str,
}
impl Parser<'_> {
    fn take(&mut self, token: &str) -> Option<()> {
        self.remaining = self.remaining.strip_prefix(token)?;
        Some(())
    }
    fn positive(&mut self) -> Option<u16> {
        let length = self
            .remaining
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        let digits = self.remaining.get(..length)?;
        if digits.starts_with('0') {
            return None;
        }
        let value = digits.parse::<u16>().ok().filter(|value| *value > 0)?;
        self.remaining = self.remaining.get(length..)?;
        Some(value)
    }
    fn count(&mut self, ceiling: u8, depth: u8) -> Option<Count> {
        if depth >= 16 {
            return None;
        }
        if self.remaining.starts_with("ternary(") {
            self.take("ternary(gte(@item.level,")?;
            let rank = u8::try_from(self.positive()?)
                .ok()
                .filter(|rank| *rank < ceiling)?;
            self.take("),")?;
            let count = self.positive()?;
            self.take(",")?;
            let otherwise = Box::new(self.count(rank, depth.checked_add(1)?)?);
            self.take(")")?;
            Some(Count::Threshold {
                rank,
                count,
                otherwise,
            })
        } else {
            self.positive().map(Count::Constant)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation_content::{
        project_presentation_content, project_record_surface_content_with_context,
        render_presentation_content_plain_text,
    };
    use crate::{ContentHash, DamagePart, RichDocument, RichNode, SpellFormId};
    const FORMULA: &str = "(ternary(gte(@item.level,8),3,ternary(gte(@item.level,6),2,1)))d6";
    fn document(formula: &str) -> RichDocument {
        RichDocument {
            nodes: vec![
                RichNode::Text {
                    text: "Before ".into(),
                },
                RichNode::Foundry {
                    node: FoundryNode::Damage {
                        formula: format!("{formula}[persistent,spirit]"),
                        options: Default::default(),
                        damage_parts: vec![DamagePart {
                            formula: formula.into(),
                            damage_type: Some("persistent,spirit".into()),
                        }],
                        label: None,
                    },
                },
                RichNode::Text {
                    text: " damage after".into(),
                },
            ],
        }
    }
    fn context(rank: u8) -> RecordSurfaceContentContext {
        RecordSurfaceContentContext::Spell {
            form_id: SpellFormId::base(
                &atlas_domain::RecordKey::parse("spells-srd:NacrNSvfODxpZena").expect("key"),
            ),
            cast_rank: rank,
        }
    }
    #[test]
    fn selected_rank_damage_preserves_all_default_inputs() {
        let document = document(FORMULA);
        let original = document.clone();
        let hash = ContentHash::for_document(&document);
        let default = project_presentation_content(&document);
        let plain = crate::render_plain_text(&document);
        let markdown = crate::render_markdown_like(&document);
        for (rank, dice) in [(2, "1d6"), (4, "1d6"), (6, "2d6"), (8, "3d6")] {
            let projected = project_record_surface_content_with_context(&document, &context(rank));
            assert!(projected.issues.is_empty());
            let text = render_presentation_content_plain_text(&projected.content);
            assert!(text.contains(&format!("{dice} persistent spirit")));
            assert!(!text.contains("ternary") && !text.contains("@item.level"));
            assert!(!text.contains("damage damage"));
        }
        assert_eq!(original, document);
        assert_eq!(hash, ContentHash::for_document(&document));
        assert_eq!(default, project_presentation_content(&document));
        assert_eq!(plain, crate::render_plain_text(&document));
        assert_eq!(markdown, crate::render_markdown_like(&document));
        assert!(plain.contains(FORMULA));
    }
    #[test]
    fn invalid_damage_is_localized_once_and_keeps_siblings() {
        for formula in [
            "((1))d6",
            "(0)d6",
            "(65536)d6",
            "(1)d0",
            "(1)d6 trailing",
            "( 1)d6",
            "(+1)d6",
        ] {
            assert!(
                Dice::parse(&format!("{formula}[persistent,spirit]")).is_none(),
                "{formula}"
            );
        }
        for formula in [
            "(ternary(gte(@actor.level,8),3,1))d6",
            "(ternary(gte(@item.level,6),2,ternary(gte(@item.level,8),3,1)))d6",
            "(ternary(gte(@item.level,8),3,(1)))d6",
            "(ternary(gte(@item.level,11),3,1))d6",
            "(ternary(gte(@item.level,8),65535,1))d65535",
            "(ternary(gte(@item.level,8),3,1))d6 trailing",
            "(ternary(gte(@item.level,8), 3,1))d6",
            "((ternary(gte(@item.level,8),3,1)))d6",
        ] {
            let projected =
                project_record_surface_content_with_context(&document(formula), &context(8));
            assert_eq!(projected.issues.len(), 1, "{formula}");
            let text = render_presentation_content_plain_text(&projected.content);
            assert!(
                text.contains("Before")
                    && text.contains("after")
                    && text.contains("Damage unavailable")
            );
            assert!(!text.contains("ternary") && !text.contains("@"));
        }
        let projected = project_record_surface_content_with_context(
            &document(FORMULA),
            &RecordSurfaceContentContext::UnavailableSpellSelection,
        );
        assert_eq!(projected.issues.len(), 1);
        assert_eq!(
            projected.issues[0].kind,
            RecordSurfaceContentIssueKind::MissingContext
        );
        let mut missing = document(FORMULA);
        if let RichNode::Foundry {
            node: FoundryNode::Damage { damage_parts, .. },
        } = &mut missing.nodes[1]
        {
            damage_parts.clear();
        }
        let projected = project_record_surface_content_with_context(&missing, &context(8));
        assert!(projected.issues.is_empty());
        assert!(
            render_presentation_content_plain_text(&projected.content)
                .contains("3d6 persistent spirit")
        );
    }
    #[test]
    fn aggregate_is_sole_evidence_and_rejects_options_and_suffixes() {
        let original = document(FORMULA);
        let expected = project_record_surface_content_with_context(&original, &context(8));
        let mut changed = original.clone();
        if let RichNode::Foundry {
            node: FoundryNode::Damage { damage_parts, .. },
        } = &mut changed.nodes[1]
        {
            *damage_parts = vec![DamagePart {
                formula: "999d99".into(),
                damage_type: Some("fire".into()),
            }];
        }
        assert_eq!(
            expected.content,
            project_record_surface_content_with_context(&changed, &context(8)).content
        );
        for suffix in [
            "[spirit,persistent]",
            "[persistent,Spirit]",
            "[persistent,spirit,fire]",
            "[persistent,-spirit]",
            "[persistent,spirit]junk",
            "[persistent,spirit ]",
            "[persistent,]",
            "",
        ] {
            let mut invalid = original.clone();
            if let RichNode::Foundry {
                node: FoundryNode::Damage { formula, .. },
            } = &mut invalid.nodes[1]
            {
                *formula = format!("{FORMULA}{suffix}");
            }
            assert_eq!(
                project_record_surface_content_with_context(&invalid, &context(8))
                    .issues
                    .len(),
                1,
                "{suffix}"
            );
        }
        if let RichNode::Foundry {
            node: FoundryNode::Damage { options, .. },
        } = &mut changed.nodes[1]
        {
            options.insert("foo".into(), "bar".into());
        }
        assert_eq!(
            project_record_surface_content_with_context(&changed, &context(8))
                .issues
                .len(),
            1
        );
    }

    #[test]
    fn ordinary_damage_never_enters_selected_evaluator() {
        let mut document = document("(1)d6");
        if let RichNode::Foundry {
            node:
                FoundryNode::Damage {
                    formula,
                    damage_parts,
                    ..
                },
        } = &mut document.nodes[1]
        {
            *formula = "(1)d6[fire]".into();
            damage_parts[0].damage_type = Some("fire".into());
        }
        let ordinary = crate::project_record_surface_content(&document);
        assert!(render_presentation_content_plain_text(&ordinary).contains("(1)d6 fire"));
        for context in [
            context(8),
            RecordSurfaceContentContext::UnavailableSpellSelection,
        ] {
            let selected = project_record_surface_content_with_context(&document, &context);
            assert!(selected.issues.is_empty());
            assert_eq!(ordinary, selected.content);
        }
        // Generic surfaces use their ordinary policy even for a dynamic node;
        // they do not invoke an evaluator and then discard its issues.
        let dynamic = self::document(FORMULA);
        assert!(
            !render_presentation_content_plain_text(&crate::project_record_surface_content(
                &dynamic
            ))
            .contains("Damage unavailable")
        );
    }

    #[test]
    fn authored_label_wins_even_without_context() {
        let mut document = document(FORMULA);
        if let RichNode::Foundry {
            node: FoundryNode::Damage { label, .. },
        } = &mut document.nodes[1]
        {
            *label = Some(vec![RichNode::Text {
                text: "Authored label".into(),
            }]);
        }
        let projected = project_record_surface_content_with_context(
            &document,
            &RecordSurfaceContentContext::UnavailableSpellSelection,
        );
        assert!(projected.issues.is_empty());
        assert!(
            render_presentation_content_plain_text(&projected.content).contains("Authored label")
        );
    }
}
