use atlas_index::FtsQuery;

#[derive(Debug, Clone, PartialEq)]
pub struct TextQueryDiagnostics {
    pub normalized_query: String,
    pub fts_query: Option<String>,
    pub fts_tokens: Vec<String>,
    pub exclude_query: Option<String>,
    pub exclude_tokens: Vec<String>,
}

pub(crate) fn normalize_record_query(value: &str) -> String {
    atlas_domain::normalize_record_name(value)
}

pub(crate) fn analyze_text_query(query: &str, exclude: Option<&str>) -> TextQueryDiagnostics {
    let normalized_query = normalize_record_query(query);
    let fts_tokens = tokenize_fts_query(query);
    let exclude_tokens = exclude.map(tokenize_fts_query).unwrap_or_default();
    let fts_query = FtsQuery::from_tokens(fts_tokens.clone()).map(|query| query.as_match_query());
    let exclude_query =
        FtsQuery::from_tokens(exclude_tokens.clone()).map(|query| query.as_match_query());
    TextQueryDiagnostics {
        normalized_query,
        fts_query,
        fts_tokens,
        exclude_query,
        exclude_tokens,
    }
}

pub(crate) fn tokenize_fts_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| !is_fts_stop_word(token))
        .map(ToString::to_string)
        .collect()
}

fn is_fts_stop_word(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "for"
            | "from"
            | "in"
            | "is"
            | "of"
            | "on"
            | "or"
            | "that"
            | "the"
            | "to"
            | "which"
            | "with"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fts_query_analysis_uses_safe_or_tokens_without_domain_derivation() {
        let analysis = analyze_text_query("monster that breathes fire", Some("water"));

        assert_eq!(analysis.fts_tokens, vec!["monster", "breathes", "fire"]);
        assert_eq!(
            analysis.fts_query.as_deref(),
            Some("\"monster\" OR \"breathes\" OR \"fire\"")
        );
        assert_eq!(analysis.exclude_tokens, vec!["water"]);
        assert_eq!(analysis.exclude_query.as_deref(), Some("\"water\""));
    }

    #[test]
    fn normalized_record_query_remains_available_to_resolution() {
        assert_eq!(normalize_record_query("Treat Wounds"), "treat wounds");
    }
}
