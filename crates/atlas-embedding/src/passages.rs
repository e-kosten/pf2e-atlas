use crate::{
    BODY_TOKEN_BUDGET, EmbeddingError, IDENTITY_TOKEN_BUDGET, OVERLAP_TOKEN_BUDGET,
    TextEmbeddingTokenizer, embedding_reuse_key, hash_document_embedding_input,
};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use std::panic::{AssertUnwindSafe, catch_unwind};
use text_splitter::{ChunkConfig, TextSplitter};

/// A selected source section. Root, owner and field attribution remains with
/// the caller; splitting never invents or merges source ownership.
#[derive(Debug, Clone, Copy)]
pub enum PassageSection<'a> {
    Identity {
        text: &'a str,
    },
    Html {
        text: &'a str,
        context: &'a str,
        prepared_html_sha256: &'a str,
        selection_version: &'a str,
        section_ordinal: usize,
        label: Option<&'a str>,
    },
    Plain {
        text: &'a str,
        context: &'a str,
        selection_version: &'a str,
        section_ordinal: usize,
        label: Option<&'a str>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedEmbeddingInput {
    pub address: SourcePassageAddress,
    pub input: String,
    pub input_sha256: String,
    pub reuse_key: String,
    pub token_count: usize,
    pub body_token_count: usize,
    pub overlap_token_count: usize,
    pub context_shortened: bool,
}

pub fn prepare_embedding_section(
    tokenizer: &TextEmbeddingTokenizer,
    section: PassageSection<'_>,
) -> Result<Vec<PreparedEmbeddingInput>, EmbeddingError> {
    let (text, context) = match section {
        PassageSection::Identity { text } => (text, ""),
        PassageSection::Html { text, context, .. }
        | PassageSection::Plain { text, context, .. } => (text, context),
    };
    if text.trim().is_empty() || tokenizer.count_tokens(text, false)? == 0 {
        return Ok(Vec::new());
    }
    let (context, context_shortened) = tokenizer.bounded_context(context)?;
    let text_hash = hash_document_embedding_input(text);
    if matches!(section, PassageSection::Identity { .. }) {
        let body_tokens = tokenizer.count_tokens(text, false)?;
        if body_tokens > IDENTITY_TOKEN_BUDGET {
            return Err(EmbeddingError::TokenBudgetExceeded {
                actual: body_tokens,
                max: IDENTITY_TOKEN_BUDGET,
            });
        }
        return Ok(vec![prepared(
            tokenizer,
            SourcePassageAddress::Identity {},
            text.to_owned(),
            body_tokens,
            0,
            false,
        )?]);
    }
    let config = ChunkConfig::new(BODY_TOKEN_BUDGET)
        .with_sizer(tokenizer.tokenizer())
        .with_trim(false)
        .with_overlap(OVERLAP_TOKEN_BUDGET)
        .map_err(|e| EmbeddingError::SplitterFailed(e.to_string()))?;
    let splitter = TextSplitter::new(config);
    // The library tokenizer sizer has an infallible trait and can panic when
    // encoding fails. Convert that failure at this boundary; never continue with
    // estimates, clipped input or a substitute splitting implementation.
    let chunks = catch_unwind(AssertUnwindSafe(|| {
        splitter.chunk_indices(text).collect::<Vec<_>>()
    }))
    .map_err(|_| EmbeddingError::SplitterFailed("tokenizer sizing failed".into()))?;
    let mut covered = 0;
    let mut previous_start = None;
    let mut output = Vec::with_capacity(chunks.len());
    for (start, body) in chunks {
        let end = start + body.len();
        if start > covered
            || end <= covered
            || previous_start.is_some_and(|previous| start <= previous)
            || text.get(start..end) != Some(body)
        {
            return Err(EmbeddingError::InvalidPassageAddress(format!(
                "nonprogressing or unfaithful range {start}..{end}"
            )));
        }
        let body_tokens = tokenizer.count_tokens(body, false)?;
        let overlap_tokens = tokenizer.count_tokens(
            text.get(start..covered)
                .ok_or_else(|| EmbeddingError::InvalidPassageAddress("invalid overlap".into()))?,
            false,
        )?;
        if body_tokens > BODY_TOKEN_BUDGET {
            return Err(EmbeddingError::TokenBudgetExceeded {
                actual: body_tokens,
                max: BODY_TOKEN_BUDGET,
            });
        }
        if overlap_tokens > OVERLAP_TOKEN_BUDGET {
            return Err(EmbeddingError::TokenBudgetExceeded {
                actual: overlap_tokens,
                max: OVERLAP_TOKEN_BUDGET,
            });
        }
        let chunk_bytes = SourceByteRange { start, end };
        let address = match section {
            PassageSection::Html {
                prepared_html_sha256,
                selection_version,
                section_ordinal,
                label,
                ..
            } => SourcePassageAddress::HtmlSection {
                prepared_html_sha256: prepared_html_sha256.into(),
                canonical_text_sha256: text_hash.clone(),
                selection_version: selection_version.into(),
                section_ordinal,
                label: label.map(str::to_owned),
                chunk_bytes,
            },
            PassageSection::Plain {
                selection_version,
                section_ordinal,
                label,
                ..
            } => SourcePassageAddress::PlainSection {
                source_text_sha256: text_hash.clone(),
                selection_version: selection_version.into(),
                section_ordinal,
                label: label.map(str::to_owned),
                chunk_bytes,
            },
            PassageSection::Identity { .. } => {
                return Err(EmbeddingError::InvalidPassageAddress(
                    "identity entered prose splitting".into(),
                ));
            }
        };
        let input = if context.is_empty() {
            body.to_owned()
        } else {
            format!("{context}\n{body}")
        };
        output.push(prepared(
            tokenizer,
            address,
            input,
            body_tokens,
            overlap_tokens,
            context_shortened,
        )?);
        covered = end;
        previous_start = Some(start);
    }
    if covered != text.len() {
        return Err(EmbeddingError::InvalidPassageAddress(format!(
            "source tail lost after byte {covered} of {}",
            text.len()
        )));
    }
    Ok(output)
}

fn prepared(
    tokenizer: &TextEmbeddingTokenizer,
    address: SourcePassageAddress,
    input: String,
    body_token_count: usize,
    overlap_token_count: usize,
    context_shortened: bool,
) -> Result<PreparedEmbeddingInput, EmbeddingError> {
    let token_count = tokenizer.validate_document_input(&input)?;
    Ok(PreparedEmbeddingInput {
        address,
        input_sha256: hash_document_embedding_input(&input),
        reuse_key: embedding_reuse_key(&input),
        input,
        token_count,
        body_token_count,
        overlap_token_count,
        context_shortened,
    })
}
