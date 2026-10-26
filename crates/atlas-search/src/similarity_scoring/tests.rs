use super::*;
use atlas_domain::{SimilarityPartSelector, SourceByteRange};

fn passages<'a>(
    seed: &[Part<'a>],
    candidate: &[Part<'a>],
    mode: SimilarityCandidateParts,
    decay: f64,
) -> Score<'a> {
    super::passages(seed, candidate, mode, decay, None).unwrap()
}
fn content_average<'a>(
    seed: &[Part<'a>],
    candidate: &[Part<'a>],
    mode: SimilarityCandidateParts,
) -> Option<Score<'a>> {
    super::content_average(seed, candidate, mode, None).unwrap()
}

fn unit(
    selector: &str,
    role: SimilarityRole,
    vector: [f32; 2],
    start: usize,
    end: usize,
) -> SourceSimilarityUnit {
    SourceSimilarityUnit {
        part: SimilarityPart {
            selector: SimilarityPartSelector(selector.into()),
            label: selector.into(),
            owner_label: "Fixture".into(),
            owners: vec![],
            field: format!("/{selector}"),
            section_ordinal: 0,
            role,
            has_vectors: true,
            label_only: false,
        },
        location: None,
        address: SourcePassageAddress::PlainSection {
            source_text_sha256: "a".repeat(64),
            selection_version: "structural-source-sections/v4".into(),
            section_ordinal: 0,
            label: None,
            chunk_bytes: SourceByteRange { start, end },
        },
        vector: vector.to_vec(),
        trait_text: None,
    }
}
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}

#[test]
fn missing_roles_keep_their_weight_and_negative_cosine_does_not_beat_missing_content() {
    let seed = vec![
        unit("description", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("reset", SimilarityRole::Reset, [0., 1.], 0, 10),
    ];
    let missing = vec![unit(
        "candidate",
        SimilarityRole::Description,
        [1., 0.],
        0,
        10,
    )];
    let negative = vec![
        unit("candidate", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("negative", SimilarityRole::Reset, [0., -1.], 0, 10),
    ];
    let seed = parts(&seed);
    let missing = parts(&missing);
    let negative = parts(&negative);
    close(
        passages(
            &seed,
            &missing,
            SimilarityCandidateParts::Corresponding,
            0.5,
        )
        .score,
        2. / 3.,
    );
    close(
        passages(
            &seed,
            &negative,
            SimilarityCandidateParts::Corresponding,
            0.5,
        )
        .score,
        2. / 3.,
    );
    close(
        passages(&seed, &missing, SimilarityCandidateParts::Corresponding, 0.).score,
        1.,
    );
    close(
        passages(&seed, &missing, SimilarityCandidateParts::Corresponding, 1.).score,
        0.5,
    );
    assert_eq!(
        passages(&seed, &missing, SimilarityCandidateParts::Corresponding, 1.).comparable_parts,
        1
    );
    assert_eq!(
        passages(
            &seed,
            &negative,
            SimilarityCandidateParts::Corresponding,
            1.
        )
        .comparable_parts,
        2
    );
}

#[test]
fn strongest_part_is_not_the_strongest_isolated_chunk_pair() {
    let seed = vec![
        unit("long", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("long", SimilarityRole::Description, [0., 1.], 10, 20),
        unit("short", SimilarityRole::Reset, [0.8, 0.6], 0, 10),
    ];
    let candidate = vec![unit(
        "candidate",
        SimilarityRole::Description,
        [1., 0.],
        0,
        10,
    )];
    let score = passages(
        &parts(&seed),
        &parts(&candidate),
        SimilarityCandidateParts::Any,
        0.,
    );
    close(score.score, 0.8);
    assert_eq!(score.parts[0].part.selector.0, "short");
    assert_eq!(score.pairs.len(), 1);
    assert_eq!(score.pairs[0].seed.part.selector.0, "short");
}

#[test]
fn overlapping_and_duplicate_ranges_do_not_add_independent_votes() {
    let mut seed = vec![
        unit("long", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("long", SimilarityRole::Description, [0., 1.], 5, 15),
    ];
    let candidate = vec![unit(
        "candidate",
        SimilarityRole::Description,
        [1., 0.],
        0,
        10,
    )];
    close(
        passages(
            &parts(&seed),
            &parts(&candidate),
            SimilarityCandidateParts::Any,
            1.,
        )
        .score,
        2. / 3.,
    );
    seed.push(seed[0].clone());
    close(
        passages(
            &parts(&seed),
            &parts(&candidate),
            SimilarityCandidateParts::Any,
            1.,
        )
        .score,
        2. / 3.,
    );
}

#[test]
fn role_centroids_obey_the_full_mask_including_other() {
    let seed = vec![
        unit("description", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("reset", SimilarityRole::Reset, [0., 1.], 0, 10),
        unit("other", SimilarityRole::Other, [1., 0.], 0, 10),
    ];
    let candidate = vec![
        unit("description", SimilarityRole::Description, [0., 1.], 0, 10),
        unit("reset", SimilarityRole::Reset, [1., 0.], 0, 10),
        unit("other", SimilarityRole::Other, [1., 0.], 0, 10),
    ];
    let seed = parts(&seed);
    let candidate = parts(&candidate);
    close(
        content_average(&seed, &candidate, SimilarityCandidateParts::Corresponding)
            .unwrap()
            .score,
        0.,
    );
    assert!(
        content_average(&seed, &candidate, SimilarityCandidateParts::Any)
            .unwrap()
            .score
            > 0.9
    );
}

#[test]
fn zero_global_centroid_is_explicitly_unavailable() {
    let seed = vec![
        unit("a", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("b", SimilarityRole::Description, [-1., 0.], 0, 10),
    ];
    let candidate = vec![unit(
        "candidate",
        SimilarityRole::Description,
        [1., 0.],
        0,
        10,
    )];
    assert!(
        content_average(
            &parts(&seed),
            &parts(&candidate),
            SimilarityCandidateParts::Any
        )
        .is_none()
    );
}

#[test]
fn candidate_role_cancellation_preserves_other_role_scores_and_denominator() {
    let seed = vec![
        unit("description", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("reset", SimilarityRole::Reset, [0., 1.], 0, 10),
    ];
    let candidate = vec![
        unit("a", SimilarityRole::Description, [1., 0.], 0, 10),
        unit("b", SimilarityRole::Description, [-1., 0.], 0, 10),
        unit("reset", SimilarityRole::Reset, [0., 1.], 0, 10),
    ];
    let score = content_average(
        &parts(&seed),
        &parts(&candidate),
        SimilarityCandidateParts::Corresponding,
    )
    .unwrap();
    close(score.score, 0.5);
    assert_eq!(score.undefined_centroids, 1);
}

#[test]
fn scoring_cancels_within_a_candidate_and_uncancelled_scores_are_identical() {
    use std::cell::Cell;
    let seed_units = vec![unit("seed", SimilarityRole::Description, [1., 0.], 0, 10)];
    let candidate_units = (0..512)
        .map(|index| {
            unit(
                "candidate",
                SimilarityRole::Description,
                [0.8, 0.6],
                index * 10,
                (index + 1) * 10,
            )
        })
        .collect::<Vec<_>>();
    let seed = parts(&seed_units);
    let candidate = parts(&candidate_units);
    let checks = Cell::new(0);
    let cancelled_after_work_starts = || {
        let count = checks.get() + 1;
        checks.set(count);
        count > 1
    };
    let error = super::passages(
        &seed,
        &candidate,
        SimilarityCandidateParts::Any,
        0.5,
        Some(&cancelled_after_work_starts),
    )
    .err()
    .unwrap();
    assert_eq!(error.kind(), crate::SearchErrorKind::OperationCancelled);
    assert_eq!(
        checks.get(),
        2,
        "cancellation must be observed inside this candidate's comparisons"
    );

    let never_cancelled = || {
        checks.set(checks.get() + 1);
        false
    };
    let expected = passages(&seed, &candidate, SimilarityCandidateParts::Any, 0.5);
    let actual = super::passages(
        &seed,
        &candidate,
        SimilarityCandidateParts::Any,
        0.5,
        Some(&never_cancelled),
    )
    .unwrap();
    assert_eq!(actual.score, expected.score);
    assert_eq!(actual.comparisons, 512);
    assert_eq!(actual.comparisons, expected.comparisons);
    assert_eq!(actual.pairs[0].cosine, expected.pairs[0].cosine);
    for mode in [
        SimilarityCandidateParts::Any,
        SimilarityCandidateParts::Corresponding,
    ] {
        checks.set(0);
        let error =
            super::content_average(&seed, &candidate, mode, Some(&cancelled_after_work_starts))
                .err()
                .unwrap();
        assert_eq!(error.kind(), crate::SearchErrorKind::OperationCancelled);
        assert_eq!(checks.get(), 2, "centroid arithmetic must also cooperate");
        let expected = content_average(&seed, &candidate, mode).unwrap();
        let actual = super::content_average(&seed, &candidate, mode, Some(&never_cancelled))
            .unwrap()
            .unwrap();
        assert_eq!(actual.score, expected.score);
        assert_eq!(actual.comparisons, expected.comparisons);
    }
}
