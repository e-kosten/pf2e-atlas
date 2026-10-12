use super::{EmbeddingError, SearchError, SearchErrorKind};

#[test]
fn empty_query_is_invalid_input_with_a_repair_instruction() {
    let error = SearchError::query_embedding(EmbeddingError::EmptyInput);

    assert_eq!(error.kind(), SearchErrorKind::InvalidOptions);
    assert_eq!(
        error.to_string(),
        "query is empty or has no body tokens; provide nonempty query text"
    );
}

#[test]
fn oversized_query_retains_exact_token_counts_and_a_repair_instruction() {
    let error = SearchError::query_embedding(EmbeddingError::TokenBudgetExceeded {
        actual: 530,
        max: 512,
    });

    assert_eq!(error.kind(), SearchErrorKind::InvalidOptions);
    assert_eq!(
        error.to_string(),
        "query exceeded embedding token budget: actual 530, max 512; shorten the query"
    );
}

#[test]
fn tokenizer_cache_model_and_inference_failures_preserve_their_failure_context() {
    let failures = [
        EmbeddingError::TokenizerLoadFailed {
            path: "tokenizer.json".into(),
            message: "unreadable".into(),
        },
        EmbeddingError::TokenizationFailed("encode failed".into()),
        EmbeddingError::ModelLoadFailed {
            path: "model.onnx".into(),
            message: "unreadable".into(),
        },
        EmbeddingError::ModelCachePrepareFailed {
            path: "model-cache".into(),
            message: "permission denied".into(),
        },
        EmbeddingError::ModelCacheDownloadFailed {
            url: "https://example.invalid/model.onnx".into(),
            path: "model.onnx".into(),
            message: "download failed".into(),
        },
        EmbeddingError::AssetChecksumMismatch {
            path: "model.onnx".into(),
            expected: "expected".into(),
            actual: "actual".into(),
        },
        EmbeddingError::UnsupportedModelContract,
        EmbeddingError::ModelRunFailed("inference failed".into()),
        EmbeddingError::DimensionMismatch {
            expected: 384,
            actual: 383,
        },
        EmbeddingError::UnexpectedEmbeddingOutputCount {
            expected: 1,
            actual: 0,
        },
        EmbeddingError::InvalidVector,
    ];

    for failure in failures {
        let context = failure.to_string();
        let error = SearchError::query_embedding(failure);

        assert_eq!(
            error.kind(),
            SearchErrorKind::EmbeddingUnavailable,
            "{context}"
        );
        assert_eq!(error.to_string(), context);
    }
}

#[test]
fn document_preparation_failures_are_internal_errors_at_the_query_boundary() {
    let failures = [
        EmbeddingError::SplitterFailed("invalid boundary".into()),
        EmbeddingError::InvalidPassageAddress("invalid address".into()),
        EmbeddingError::PreparedInputMismatch,
    ];

    for failure in failures {
        let context = failure.to_string();
        let error = SearchError::query_embedding(failure);

        assert_eq!(error.kind(), SearchErrorKind::QueryFailed, "{context}");
        assert_eq!(error.to_string(), context);
    }
}
