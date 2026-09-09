use atlas_app_model::{
    H8FactView, H8IdentityStabilityView, H8PageSourceMetadataView, H8ProvenanceView,
    H8SourceMetadataView, H8UnsupportedChildView, H8UnsupportedFieldView, H8UnsupportedValueView,
    JournalPageEntryView, JournalPageTextView, JournalPageTitleView, JournalPageVideoView,
    JournalPageView, JournalSurfaceView, RollTableSurfaceView, TableResultEntryView,
    TableResultRangeView, TableResultSourceMetadataView, TableResultView, TableRollCapabilityView,
    TableRollSubjectView, TableRollUnavailableReasonView, TableRollUnavailableView,
};
use atlas_record::{
    ContentChildIdentity, ContentChildLocator, FactValue, H8ExactSourceObject, H8Fact,
    H8FieldValue, H8Number, H8Provenance, H8RecordSourceMetadata, H8UnsupportedChild,
    H8UnsupportedField, JournalPage, JournalPageEntry, JournalPageKind, JournalPageText,
    JournalPageTitle, JournalPageVideo, JournalRecord, RichDocument, RollTableRecord, TableResult,
    TableResultEntry, TableResultKind, TableRollUnavailable, TableRollUnavailableReason,
    UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
    encode_content_child_selector, project_record_surface_content,
};

pub(crate) fn journal_surface(value: &JournalRecord) -> JournalSurfaceView {
    JournalSurfaceView {
        source_id: value.identity.source_id.as_str().to_string(),
        pages: fact(&value.pages, |pages| {
            pages.iter().map(journal_page_entry).collect()
        }),
        content: value
            .content
            .documents
            .iter()
            .filter_map(super::surface::content_view)
            .collect(),
        source_metadata: source_metadata(&value.source_metadata),
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
        provenance: provenance(&value.provenance),
    }
}

pub(crate) fn roll_table_surface(value: &RollTableRecord) -> RollTableSurfaceView {
    RollTableSurfaceView {
        source_id: value.identity.source_id.as_str().to_string(),
        description: fact(&value.description, content_blocks),
        results: fact(&value.results, |results| {
            results.iter().map(table_result_entry).collect()
        }),
        formula: fact(&value.formula, Clone::clone),
        replacement: fact(&value.replacement, |value| *value),
        display_roll: fact(&value.display_roll, |value| *value),
        image: fact(&value.image, |value| value.as_str().to_string()),
        roll: table_roll_capability(value),
        content: value
            .content
            .documents
            .iter()
            .filter_map(super::surface::content_view)
            .collect(),
        source_metadata: source_metadata(&value.source_metadata),
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
        provenance: provenance(&value.provenance),
    }
}

fn journal_page_entry(value: &JournalPageEntry) -> JournalPageEntryView {
    match value {
        JournalPageEntry::Page(value) => JournalPageEntryView::Page {
            page: Box::new(journal_page(value)),
        },
        JournalPageEntry::Unsupported(value) => JournalPageEntryView::Unsupported {
            unsupported: unsupported_child(value),
        },
    }
}

fn journal_page(value: &JournalPage) -> JournalPageView {
    JournalPageView {
        locator: encode_content_child_selector(&value.locator),
        identity_stability: identity_stability(&value.locator),
        source_id: fact(&value.source_id, |value| value.as_str().to_string()),
        source_ordinal: value.source_ordinal,
        name: fact(&value.name, Clone::clone),
        page_kind: fact(&value.page_kind, |value| match value {
            JournalPageKind::Text => "text".to_string(),
            JournalPageKind::Image => "image".to_string(),
            JournalPageKind::Pdf => "pdf".to_string(),
            JournalPageKind::Video => "video".to_string(),
        }),
        sort: fact(&value.sort, |value| *value),
        title: fact(&value.title, title),
        text: fact(&value.text, text),
        source: fact(&value.source, |value| value.as_str().to_string()),
        image_source: exact_object(&value.image_source),
        image_caption: fact(&value.image_caption, Clone::clone),
        video: fact(&value.video, video),
        source_system: fact(&value.source_system, |value| value.compact_json.clone()),
        source_metadata: H8PageSourceMetadataView {
            ownership: exact_object(&value.source_metadata.ownership),
            flags: exact_object(&value.source_metadata.flags),
            stats: exact_object(&value.source_metadata.stats),
        },
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
    }
}

fn title(value: &JournalPageTitle) -> JournalPageTitleView {
    JournalPageTitleView {
        show: fact(&value.show, |value| *value),
        level: fact(&value.level, |value| *value),
    }
}

fn text(value: &JournalPageText) -> JournalPageTextView {
    JournalPageTextView {
        content: fact(&value.content, content_blocks),
        format: fact(&value.format, |value| *value),
        markdown: fact(&value.markdown, Clone::clone),
    }
}

fn video(value: &JournalPageVideo) -> JournalPageVideoView {
    JournalPageVideoView {
        controls: fact(&value.controls, |value| *value),
        loop_playback: fact(&value.loop_playback, |value| *value),
        autoplay: fact(&value.autoplay, |value| *value),
        volume: number(&value.volume),
        timestamp: number(&value.timestamp),
        width: number(&value.width),
        height: number(&value.height),
    }
}

fn table_result_entry(value: &TableResultEntry) -> TableResultEntryView {
    match value {
        TableResultEntry::Result(value) => TableResultEntryView::Result {
            result: Box::new(table_result(value)),
        },
        TableResultEntry::Unsupported(value) => TableResultEntryView::Unsupported {
            unsupported: unsupported_child(value),
        },
    }
}

pub(crate) fn table_result(value: &TableResult) -> TableResultView {
    TableResultView {
        locator: encode_content_child_selector(&value.locator),
        identity_stability: identity_stability(&value.locator),
        source_id: fact(&value.source_id, |value| value.as_str().to_string()),
        source_ordinal: value.source_ordinal,
        result_kind: fact(&value.result_kind, |value| match value {
            TableResultKind::Text => "text".to_string(),
            TableResultKind::Pack => "pack".to_string(),
            TableResultKind::Document => "document".to_string(),
        }),
        text: fact(&value.text, content_blocks),
        collection: fact(&value.target.collection, Clone::clone),
        document_id: fact(&value.target.document_id, |value| {
            value.as_str().to_string()
        }),
        weight: number(&value.weight),
        range: fact(&value.range, |value| TableResultRangeView {
            first: value.first,
            last: value.last,
        }),
        drawn: fact(&value.drawn, |value| *value),
        image: fact(&value.image, |value| value.as_str().to_string()),
        source_metadata: TableResultSourceMetadataView {
            flags: exact_object(&value.source_metadata.flags),
        },
        unsupported_fields: value
            .unsupported_fields
            .iter()
            .map(unsupported_field)
            .collect(),
    }
}

pub(crate) fn table_roll_capability(value: &RollTableRecord) -> TableRollCapabilityView {
    match value.validate_rollability() {
        Ok(table) => TableRollCapabilityView::Available {
            formula: table.normalized_formula(),
            sides: table.sides().get(),
        },
        Err(unavailable) => TableRollCapabilityView::Unavailable {
            unavailable: table_roll_unavailable(&unavailable),
        },
    }
}

pub(crate) fn table_roll_unavailable(value: &TableRollUnavailable) -> TableRollUnavailableView {
    let reason = match value.reason {
        TableRollUnavailableReason::FormulaMissing => {
            TableRollUnavailableReasonView::FormulaMissing
        }
        TableRollUnavailableReason::FormulaNull => TableRollUnavailableReasonView::FormulaNull,
        TableRollUnavailableReason::FormulaUnsupported => {
            TableRollUnavailableReasonView::FormulaUnsupported
        }
        TableRollUnavailableReason::FormulaInvalid => {
            TableRollUnavailableReasonView::FormulaInvalid
        }
        TableRollUnavailableReason::ResultsMissing => {
            TableRollUnavailableReasonView::ResultsMissing
        }
        TableRollUnavailableReason::ResultsNull => TableRollUnavailableReasonView::ResultsNull,
        TableRollUnavailableReason::ResultsUnsupported => {
            TableRollUnavailableReasonView::ResultsUnsupported
        }
        TableRollUnavailableReason::UnsupportedResult => {
            TableRollUnavailableReasonView::UnsupportedResult
        }
        TableRollUnavailableReason::RangeMissing => TableRollUnavailableReasonView::RangeMissing,
        TableRollUnavailableReason::RangeNull => TableRollUnavailableReasonView::RangeNull,
        TableRollUnavailableReason::RangeUnsupported => {
            TableRollUnavailableReasonView::RangeUnsupported
        }
        TableRollUnavailableReason::RangeOutOfDomain => {
            TableRollUnavailableReasonView::RangeOutOfDomain
        }
    };
    let subject = match (&value.child_locator, value.source_ordinal) {
        (Some(locator), Some(source_ordinal)) => TableRollSubjectView::Result {
            locator: encode_content_child_selector(locator),
            source_ordinal,
        },
        _ => TableRollSubjectView::Table,
    };
    TableRollUnavailableView {
        reason,
        subject,
        message: table_roll_unavailable_message(value),
    }
}

fn table_roll_unavailable_message(value: &TableRollUnavailable) -> String {
    let detail = match value.reason {
        TableRollUnavailableReason::FormulaMissing => "the formula is missing",
        TableRollUnavailableReason::FormulaNull => "the formula is null",
        TableRollUnavailableReason::FormulaUnsupported => "the formula is unsupported",
        TableRollUnavailableReason::FormulaInvalid => "the formula is not an exact 1dN expression",
        TableRollUnavailableReason::ResultsMissing => "the result collection is missing",
        TableRollUnavailableReason::ResultsNull => "the result collection is null",
        TableRollUnavailableReason::ResultsUnsupported => "the result collection is unsupported",
        TableRollUnavailableReason::UnsupportedResult => "a result is unsupported",
        TableRollUnavailableReason::RangeMissing => "a result range is missing",
        TableRollUnavailableReason::RangeNull => "a result range is null",
        TableRollUnavailableReason::RangeUnsupported => "a result range is unsupported",
        TableRollUnavailableReason::RangeOutOfDomain => {
            "a result range falls outside the formula domain"
        }
    };
    match value.source_ordinal {
        Some(source_ordinal) => format!(
            "Roll is unavailable because result {} is incomplete or invalid: {detail}.",
            u64::from(source_ordinal) + 1
        ),
        None => format!("Roll is unavailable because {detail}."),
    }
}

fn content_blocks(value: &RichDocument) -> Vec<atlas_app_model::CreatureSurfaceContentBlockView> {
    super::surface::project_content(project_record_surface_content(value))
}

fn source_metadata(value: &H8RecordSourceMetadata) -> H8SourceMetadataView {
    H8SourceMetadataView {
        folder: fact(&value.folder, |value| value.as_str().to_string()),
        sort: fact(&value.sort, |value| *value),
        ownership: exact_object(&value.ownership),
        flags: exact_object(&value.flags),
        stats: exact_object(&value.stats),
    }
}

fn provenance(value: &H8Provenance) -> H8ProvenanceView {
    H8ProvenanceView {
        source_path: value.source_path.clone(),
        source_contract_version: value.source_contract_version.clone(),
        source_system_version: value.source_system_version.clone(),
        source_upstream_commit: value.source_upstream_commit.clone(),
    }
}

fn unsupported_child(value: &H8UnsupportedChild) -> H8UnsupportedChildView {
    H8UnsupportedChildView {
        locator: encode_content_child_selector(&value.locator),
        identity_stability: identity_stability(&value.locator),
        source_id: fact(&value.source_id, |value| value.as_str().to_string()),
        source_ordinal: value.source_ordinal,
        exact_source: value.exact_source.compact_json.clone(),
        reason: value.reason.clone(),
    }
}

fn identity_stability(value: &ContentChildLocator) -> H8IdentityStabilityView {
    match value.identity {
        ContentChildIdentity::Stable(_) => H8IdentityStabilityView::StableSourceId,
        ContentChildIdentity::Unstable { .. } => H8IdentityStabilityView::UnstableAuthoredOrdinal,
    }
}

fn exact_object(value: &H8Fact<H8ExactSourceObject>) -> H8FactView<String> {
    fact(value, |value| value.compact_json.clone())
}

fn number(value: &H8Fact<H8Number>) -> H8FactView<String> {
    fact(value, |value| value.canonical.clone())
}

fn fact<T, U>(value: &H8Fact<T>, map: impl FnOnce(&T) -> U) -> H8FactView<U> {
    match value {
        FactValue::Missing => H8FactView::Missing,
        FactValue::Null => H8FactView::Null,
        FactValue::Value(H8FieldValue::Known(value)) => H8FactView::Known(map(value)),
        FactValue::Value(H8FieldValue::Unsupported(value)) => {
            H8FactView::Unsupported(unsupported_value(value))
        }
    }
}

fn unsupported_field(value: &H8UnsupportedField) -> H8UnsupportedFieldView {
    H8UnsupportedFieldView {
        relative_path: value.relative_path.clone(),
        authored_order: value.authored_order,
        value: unsupported_value(&value.value),
    }
}

fn unsupported_value(value: &UnsupportedSourceValue) -> H8UnsupportedValueView {
    H8UnsupportedValueView {
        shape: match value.shape {
            UnsupportedSourceShape::Missing => "missing",
            UnsupportedSourceShape::Null => "null",
            UnsupportedSourceShape::String => "string",
            UnsupportedSourceShape::Number => "number",
            UnsupportedSourceShape::Boolean => "boolean",
            UnsupportedSourceShape::Array => "array",
            UnsupportedSourceShape::Object => "object",
        }
        .to_string(),
        exact_value: value.value.clone(),
        reason: match value.reason {
            UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
            UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
            UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
            UnsupportedSourceReason::NonCanonicalRuntimeValue => "non_canonical_runtime_value",
            UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
        }
        .to_string(),
    }
}
