use crate::{
    BODY_TOKEN_BUDGET, EmbeddingError, OVERLAP_TOKEN_BUDGET, PassageSection,
    PreparedEmbeddingInput, TextEmbeddingTokenizer, hash_document_embedding_input,
};
use atlas_domain::SourcePassageAddress;

/// Published layout evidence; text and token counts are always derived again.
#[derive(Debug, Clone)]
pub struct PassageLayout {
    pub address: SourcePassageAddress,
    pub input_sha256: String,
}

/// Replay a recipe-compatible layout against current text. A miss requires normal
/// splitting; partial coverage or changed context never supplies partial output.
pub fn reuse_embedding_section(
    tokenizer: &TextEmbeddingTokenizer,
    section: PassageSection<'_>,
    layouts: &[PassageLayout],
) -> Result<Option<Vec<PreparedEmbeddingInput>>, EmbeddingError> {
    if layouts.is_empty() || matches!(section, PassageSection::Identity { .. }) {
        return Ok(None);
    }
    let (text, context) = section.text_and_context();
    let text_hash = hash_document_embedding_input(text);
    let (context, shortened) = tokenizer.bounded_context(context)?;
    let mut covered = 0;
    let mut previous_start = None;
    let mut output = Vec::with_capacity(layouts.len());
    for layout in layouts {
        let range = match &layout.address {
            SourcePassageAddress::HtmlSection { chunk_bytes, .. }
            | SourcePassageAddress::PlainSection { chunk_bytes, .. } => *chunk_bytes,
            SourcePassageAddress::Identity {} => return Ok(None),
        };
        if range.start > covered
            || range.end <= covered
            || previous_start.is_some_and(|start| range.start <= start)
            || section.address(&text_hash, range).as_ref() != Some(&layout.address)
        {
            return Ok(None);
        }
        let (Some(body), Some(overlap)) = (
            text.get(range.start..range.end),
            text.get(range.start..covered),
        ) else {
            return Ok(None);
        };
        let input = if context.is_empty() {
            body.to_owned()
        } else {
            format!("{context}\n{body}")
        };
        if hash_document_embedding_input(&input) != layout.input_sha256 {
            return Ok(None);
        }
        let body_tokens = tokenizer.count_tokens(body, false)?;
        let overlap_tokens = tokenizer.count_tokens(overlap, false)?;
        if body_tokens > BODY_TOKEN_BUDGET || overlap_tokens > OVERLAP_TOKEN_BUDGET {
            return Ok(None);
        }
        output.push(crate::passages::prepared(
            tokenizer,
            layout.address.clone(),
            input,
            body_tokens,
            overlap_tokens,
            shortened,
        )?);
        covered = range.end;
        previous_start = Some(range.start);
    }
    Ok((covered == text.len()).then_some(output))
}
