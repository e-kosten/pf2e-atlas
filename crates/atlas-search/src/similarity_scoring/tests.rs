use super::*;
use atlas_domain::{SimilarityPart, SimilarityPartSelector, SimilarityRole, SourceByteRange};

fn parts(units: &[SourceSimilarityUnit]) -> Vec<Part<'_>> {
    super::parts(units).unwrap()
}

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
        component: SimilarityComponent::Section {
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
    assert_eq!(score.parts[0].part.selector().0, "short");
    assert_eq!(score.pairs.len(), 1);
    assert_eq!(score.pairs[0].seed.component.selector().0, "short");
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

#[test]
fn reused_maxima_reselect_evidence_after_decay_when_tied_parts_have_different_peak_chunks() {
    let mut source = vec![];
    for (selector, values) in [
        ("a", [0.5_f32, 0.5]),
        ("b", [0.75, 0.25]),
        ("c", [1.0, 0.0]),
        ("d", [0.625, 0.375]),
    ] {
        for (index, cosine) in values.into_iter().enumerate() {
            source.push(unit(
                selector,
                SimilarityRole::Description,
                [cosine, (1.0 - cosine * cosine).sqrt()],
                index * 10,
                (index + 1) * 10,
            ));
        }
    }
    let target = [unit(
        "target",
        SimilarityRole::Description,
        [1.0, 0.0],
        0,
        10,
    )];
    let seed = parts(&source);
    let candidate = parts(&target);
    let maxima =
        super::passage_maxima(&seed, &candidate, SimilarityCandidateParts::Any, None).unwrap();
    assert_eq!(maxima.pairs.len(), 4);
    // At equal weights the three largest peak cosines omit part a, but at
    // decay zero its stable selector makes it the only actual contributor.
    assert!(
        !passages(&seed, &candidate, SimilarityCandidateParts::Any, 1.0)
            .pairs
            .iter()
            .any(|pair| pair.seed.component.selector().0 == "a")
    );
    let strongest = passages(&seed, &candidate, SimilarityCandidateParts::Any, 0.0);
    close(strongest.score, 0.5);
    assert_eq!(strongest.pairs.len(), 1);
    assert_eq!(strongest.pairs[0].seed.component.selector().0, "a");
    for decay in [0.0, 0.25, 0.5, 1.0] {
        let weights = super::decay_weights(maxima.parts.len(), decay);
        let chosen = super::select_passage_pairs(
            &maxima.pairs,
            |pair| {
                maxima
                    .parts
                    .iter()
                    .zip(&weights)
                    .find(|(part, _)| part.part.selector() == pair.seed.component.selector())
                    .map_or(0.0, |(part, weight)| part.score * weight)
            },
            3,
        );
        let direct = passages(&seed, &candidate, SimilarityCandidateParts::Any, decay);
        assert_eq!(
            chosen
                .iter()
                .map(|pair| (pair.seed.component.selector(), pair.cosine))
                .collect::<Vec<_>>(),
            direct
                .pairs
                .iter()
                .map(|pair| (pair.seed.component.selector(), pair.cosine))
                .collect::<Vec<_>>()
        );
        close(
            maxima
                .parts
                .iter()
                .zip(&weights)
                .map(|(part, weight)| part.score * weight)
                .sum(),
            direct.score,
        );
    }
}

fn identity(selector: &str, vector: [f32; 2]) -> SourceSimilarityUnit {
    SourceSimilarityUnit {
        component: SimilarityComponent::Identity {
            selector: SimilarityPartSelector(selector.into()),
            label: "Name and identity".into(),
            owner_label: "Fixture".into(),
            has_vectors: true,
        },
        location: None,
        address: SourcePassageAddress::Identity {},
        vector: vector.to_vec(),
        trait_text: None,
    }
}

#[test]
fn frozen_identity_and_description_scores_use_one_weight_per_logical_component() {
    let seed = vec![
        identity("identity", [1., 0.]),
        unit("description", SimilarityRole::Description, [0., 1.], 0, 10),
        unit("description", SimilarityRole::Description, [0., 1.], 5, 15),
    ];
    let candidate = vec![
        identity("candidate-identity", [1., 0.]),
        unit(
            "candidate-description",
            SimilarityRole::Description,
            [0.6, 0.8],
            0,
            10,
        ),
    ];
    let seed = parts(&seed);
    let candidate = parts(&candidate);
    assert_eq!(seed.len(), 2);
    assert_eq!(
        seed.iter()
            .find(|part| part.metadata.is_identity())
            .unwrap()
            .weights,
        vec![1.]
    );
    for mode in [
        SimilarityCandidateParts::Any,
        SimilarityCandidateParts::Corresponding,
    ] {
        for (decay, expected) in [(0., 1.), (0.5, 14. / 15.), (1., 0.9)] {
            close(passages(&seed, &candidate, mode, decay).score, expected);
        }
    }
    close(
        content_average(&seed, &candidate, SimilarityCandidateParts::Any)
            .unwrap()
            .score,
        0.9486832980505138,
    );
    close(
        content_average(&seed, &candidate, SimilarityCandidateParts::Corresponding)
            .unwrap()
            .score,
        0.9,
    );
}

#[test]
fn frozen_cross_component_matches_require_any_alignment() {
    let seed = vec![
        identity("identity", [1., 0.]),
        unit("description", SimilarityRole::Description, [0., 1.], 0, 10),
    ];
    let candidate = vec![
        identity("candidate-identity", [0., 1.]),
        unit(
            "candidate-description",
            SimilarityRole::Description,
            [1., 0.],
            0,
            10,
        ),
    ];
    let seed = parts(&seed);
    let candidate = parts(&candidate);
    for (mode, expected) in [
        (SimilarityCandidateParts::Any, 1.),
        (SimilarityCandidateParts::Corresponding, 0.),
    ] {
        close(passages(&seed, &candidate, mode, 0.5).score, expected);
        close(
            content_average(&seed, &candidate, mode).unwrap().score,
            expected,
        );
    }
}

#[test]
fn missing_description_does_not_renormalize_identity_contribution() {
    let seed = vec![
        identity("identity", [1., 0.]),
        unit("description", SimilarityRole::Description, [0., 1.], 0, 10),
    ];
    let candidate = vec![identity("candidate-identity", [1., 0.])];
    let seed = parts(&seed);
    let candidate = parts(&candidate);
    let passage = passages(
        &seed,
        &candidate,
        SimilarityCandidateParts::Corresponding,
        0.5,
    );
    close(passage.score, 2. / 3.);
    assert_eq!(passage.comparable_parts, 1);
    let average =
        content_average(&seed, &candidate, SimilarityCandidateParts::Corresponding).unwrap();
    close(average.score, 0.5);
    assert_eq!(average.comparable_parts, 1);
}

#[test]
fn identity_only_is_decay_invariant_and_keeps_the_positive_floor() {
    let seed = vec![identity("identity", [1., 0.])];
    let positive = vec![identity("candidate", [0.6, 0.8])];
    let negative = vec![identity("candidate", [-1., 0.])];
    let seed = parts(&seed);
    for (units, expected) in [(&positive, 0.6), (&negative, 0.)] {
        let candidate = parts(units);
        for mode in [
            SimilarityCandidateParts::Any,
            SimilarityCandidateParts::Corresponding,
        ] {
            for decay in [0., 0.25, 0.5, 0.85, 1.] {
                close(passages(&seed, &candidate, mode, decay).score, expected);
            }
            close(
                content_average(&seed, &candidate, mode).unwrap().score,
                expected,
            );
        }
    }
}

#[test]
fn duplicate_identity_and_fake_section_identity_addresses_are_rejected() {
    let duplicate = vec![
        identity("identity", [1., 0.]),
        identity("second-identity", [1., 0.]),
    ];
    assert_eq!(
        super::parts(&duplicate).err().unwrap().kind(),
        crate::SearchErrorKind::ArtifactContractViolation
    );
    let mut fake = unit("fake", SimilarityRole::Description, [1., 0.], 0, 10);
    fake.address = SourcePassageAddress::Identity {};
    assert_eq!(
        super::parts(&[fake]).err().unwrap().kind(),
        crate::SearchErrorKind::ArtifactContractViolation
    );
}

#[test]
fn undefined_global_identity_plus_prose_average_never_becomes_identity_fallback() {
    let seed = vec![
        identity("identity", [1., 0.]),
        unit("description", SimilarityRole::Description, [-1., 0.], 0, 10),
    ];
    let candidate = vec![identity("candidate", [1., 0.])];
    assert!(
        content_average(
            &parts(&seed),
            &parts(&candidate),
            SimilarityCandidateParts::Any
        )
        .is_none()
    );
    close(
        passages(
            &parts(&seed),
            &parts(&candidate),
            SimilarityCandidateParts::Corresponding,
            0.5,
        )
        .score,
        2. / 3.,
    );
}
