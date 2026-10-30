use super::{
    filters::{resolve_filter, resolve_reference},
    product::{
        app_result, print_relationship_evidence, print_search_matches, print_summary,
        report_app_error,
    },
};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    AppError, AppErrorCode, SearchCriteria, SearchModeSummary, SearchPageRequest,
    SearchResultsPage, SearchResultsRequest, SimilarityEvidenceView, SimilarityPartsRequest,
};
use atlas_cli_support::{write_json_data, write_json_error};
use atlas_domain::{
    SimilarityCandidateParts, SimilarityOptions, SimilarityPartSelector, SimilaritySeed,
};
use std::process::ExitCode;
pub(crate) mod args;
mod recovery;
use args::{CliCandidateParts, CliSimilarityMethod, SimilarOptions};

pub(crate) fn run_similar(o: SimilarOptions) -> Result<ExitCode, String> {
    if let Some(message) = recovery::missing_generation(&o) {
        if o.json {
            write_json_error("invalid_input", message)?;
        } else {
            eprintln!("{message}");
        }
        return Ok(ExitCode::from(2));
    }
    let options = match comparison_options(&o) {
        Ok(options) => options,
        Err(error) => return report_app_error(error, o.json),
    };
    if o.list_parts && has_candidate_filters(&o.filter_options) {
        return report_app_error(
            invalid(
                "--list-parts does not accept candidate filters; list seed parts first, then apply filters to a Similar search",
            ),
            o.json,
        );
    }
    let sort = super::result_order::requested_sort(&o.sort, o.seed)?;
    let service = match connect(ClientOptions {
        search_profile: o.search_profile.clone(),
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index.clone(),
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let filter = if o.list_parts {
        None
    } else {
        match resolve_filter(&o.filter_options, &service) {
            Ok(filter) => filter,
            Err(error) => return report_app_error(error, o.json),
        }
    };
    let seed = if o.record_ref.starts_with("trait:") {
        match o.record_ref.parse() {
            Ok(trait_key) => SimilaritySeed::Trait { trait_key },
            Err(error) => return report_app_error(invalid(format!("{error}")), o.json),
        }
    } else {
        match resolve_reference(&service, &o.record_ref) {
            Ok(record_key) => SimilaritySeed::Record { record_key },
            Err(error) => return report_app_error(error, o.json),
        }
    };
    let rediscovery = recovery::command(&o, Some(&seed));
    let explicit_parts = !o.parts.is_empty();
    let options = options.unwrap_or_else(|| SimilarityOptions::default_for_seed(&seed));
    if let Err(error) = options.validate(&seed) {
        return report_app_error(invalid(error), o.json);
    }
    if o.list_parts {
        let mut inventory = match app_result(
            service.similarity_parts(
                SimilarityPartsRequest {
                    seed,
                    search_profile: o.search_profile,
                    expected_artifact_generation: None,
                },
                &atlas_app_service::ReadExecutionContext::uncancellable(),
            ),
            o.json,
        )? {
            std::ops::ControlFlow::Continue(value) => value,
            std::ops::ControlFlow::Break(exit) => return Ok(exit),
        };
        if matches!(o.sections, Some(crate::cli::args::CliSearchContent::Core)) {
            inventory
                .parts
                .retain(|part| inventory.core.selectors.contains(part.selector()));
        }
        if o.json {
            write_json_data(&inventory)?;
        } else {
            println!("Comparison content for {}", o.record_ref);
            println!("Generation: {}", inventory.artifact_generation);
            println!("Model: {}\n", inventory.search_profile);
            if matches!(o.sections, Some(crate::cli::args::CliSearchContent::Core)) {
                println!(
                    "Core selection: {}\n",
                    core_representation_label(inventory.core.representation)
                );
            }
            for part in inventory.parts {
                println!(
                    "{} / {}{}",
                    part.owner_label(),
                    part.label(),
                    if part.has_vectors() {
                        ""
                    } else {
                        " (vectors unavailable)"
                    }
                );
                if matches!(&part, atlas_app_model::SimilarityComponentView::Section { part } if part.label_only)
                {
                    println!("  Label only; no definition text");
                }
                println!("  --part '{}'", part.selector().0);
            }
            println!(
                "\nUse --artifact-generation '{}' with explicit parts.",
                inventory.artifact_generation
            );
        }
        return Ok(ExitCode::SUCCESS);
    }
    let criteria = match seed {
        SimilaritySeed::Record { record_key } => SearchCriteria::SimilarRecords {
            source_sections: o.sections.map(Into::into),
            search_content: o.search_content.map(Into::into),
            seed_record_key: record_key.to_string(),
            filter,
            similarity: Some(options),
        },
        SimilaritySeed::Trait { trait_key } => SearchCriteria::SimilarTraits {
            source_sections: o.sections.map(Into::into),
            search_content: o.search_content.map(Into::into),
            seed_trait_key: trait_key.to_string(),
            filter,
            similarity: Some(options),
        },
    };
    let result = match app_result(
        service
            .search_page(
                SearchResultsRequest {
                    sort,
                    ranking: None,
                    search_profile: o.search_profile,
                    criteria,
                    page: SearchPageRequest {
                        number: o.page,
                        size: o.limit,
                    },
                    expected_artifact_generation: o.artifact_generation,
                },
                &atlas_app_service::ReadExecutionContext::uncancellable(),
            )
            .map_err(|error| {
                recovery::enrich(error.into_app_error(), &rediscovery, explicit_parts).into()
            }),
        o.json,
    )? {
        std::ops::ControlFlow::Continue(value) => value,
        std::ops::ControlFlow::Break(exit) => return Ok(exit),
    };
    if o.json {
        write_json_data(&result)?;
    } else {
        print_results(&result, o.explain);
    }
    Ok(ExitCode::SUCCESS)
}

fn comparison_options(o: &SimilarOptions) -> Result<Option<SimilarityOptions>, AppError> {
    let parts = (!o.parts.is_empty()).then(|| {
        o.parts
            .iter()
            .cloned()
            .map(SimilarityPartSelector)
            .collect()
    });
    let candidate_parts = match o.candidate_parts.unwrap_or(CliCandidateParts::Any) {
        CliCandidateParts::Any => SimilarityCandidateParts::Any,
        CliCandidateParts::Corresponding => SimilarityCandidateParts::Corresponding,
    };
    let method = o.method.or_else(|| {
        (o.decay.is_some()
            || parts.is_some()
            || o.candidate_parts.is_some()
            || o.sections.is_some())
        .then_some(CliSimilarityMethod::Passages)
    });
    let Some(method) = method else {
        return Ok(None);
    };
    match method {
        CliSimilarityMethod::Passages => Ok(SimilarityOptions::Passages {
            decay: o
                .decay
                .unwrap_or_else(atlas_domain::default_similarity_decay),
            parts,
            candidate_parts,
        }),
        CliSimilarityMethod::ContentAverage => {
            if o.decay.is_some() {
                return Err(invalid("--decay applies only to --method passages"));
            }
            Ok(SimilarityOptions::ContentAverage {
                parts,
                candidate_parts,
            })
        }
        CliSimilarityMethod::Baseline => {
            if o.decay.is_some() || parts.is_some() || o.candidate_parts.is_some() || o.sections.is_some() {
                return Err(invalid(
                    "--method baseline does not accept --decay, --part, --sections or --candidate-parts",
                ));
            }
            Ok(SimilarityOptions::Baseline {})
        }
    }
    .map(Some)
}

fn has_candidate_filters(filters: &crate::cli::args::FilterOptions) -> bool {
    filters.where_expression.is_some()
        || [
            &filters.kinds,
            &filters.pack_names,
            &filters.rarities,
            &filters.traits,
            &filters.links_to,
            &filters.linked_from,
            &filters.links_to_main_entry,
            &filters.linked_from_main_entry,
        ]
        .iter()
        .any(|values| !values.is_empty())
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::new(AppErrorCode::InvalidRequest, message)
}

fn print_results(result: &SearchResultsPage, explain: bool) {
    match &result.mode {
        SearchModeSummary::SimilarRecords { seed } => println!("Similar to {}", seed.title),
        SearchModeSummary::SimilarTraits { title, .. } => println!("Similar to {title}"),
        _ => {}
    }
    if let Some(configuration) = &result.similarity {
        let method = match &configuration.options {
            SimilarityOptions::Passages {
                decay,
                candidate_parts,
                ..
            } => {
                format!(
                    "Passage matches; decay {decay}; {}",
                    candidate_mode_label(*candidate_parts)
                )
            }
            SimilarityOptions::ContentAverage {
                candidate_parts, ..
            } => {
                format!(
                    "Content average; {}",
                    candidate_mode_label(*candidate_parts)
                )
            }
            SimilarityOptions::Baseline {} => {
                if matches!(result.mode, SearchModeSummary::SimilarTraits { .. }) {
                    "Trait definition"
                } else {
                    "Name and identity"
                }
                .into()
            }
        };
        println!("{method}; model {}\n", result.search_profile);
        if let Some(source) = configuration.source_sections {
            println!(
                "Source selection: {}",
                match source {
                    atlas_domain::SimilaritySourceSections::All => "All sections and identity",
                    atlas_domain::SimilaritySourceSections::Core => "Core sections",
                    atlas_domain::SimilaritySourceSections::Custom => "Custom selection",
                }
            );
        }
        println!(
            "Search content: {}",
            match configuration.search_content {
                atlas_domain::SearchContentScope::All => "All sections",
                atlas_domain::SearchContentScope::Core => "Core sections",
            }
        );
        if let Some(core) = &configuration.core {
            println!(
                "Core selection: {}",
                core_representation_label(core.representation)
            );
        }
        println!();
        if explain && matches!(configuration.options, SimilarityOptions::Baseline {}) {
            println!(
                "Seed representation: {}",
                if matches!(result.mode, SearchModeSummary::SimilarTraits { .. }) {
                    "stored trait definition average"
                } else {
                    "stored name and identity vector"
                }
            );
        }
        if configuration.diagnostics.undefined_candidate_centroids > 0 {
            println!(
                "{} candidate content averages were undefined",
                configuration.diagnostics.undefined_candidate_centroids
            );
        }
    }
    super::result_order::print_random_seed(&result.sort);
    for row in &result.rows {
        if let Some(evidence) = &row.similarity {
            print_summary(&row.record);
            print_evidence(evidence, explain);
            print_relationship_evidence(&row.relationship_evidence);
            println!();
        } else {
            print_search_matches(std::slice::from_ref(row));
        }
    }
    super::product::print_search_page_count(result);
}

fn print_evidence(evidence: &SimilarityEvidenceView, explain: bool) {
    if explain {
        println!(
            "  Score {:.4}; {} of {} parts comparable",
            evidence.score, evidence.comparable_parts, evidence.selected_parts
        );
        for part in &evidence.parts {
            println!(
                "  {} / {}: score {:.4}, weight {:.4}{}",
                part.part.owner_label(),
                part.part.label(),
                part.score,
                part.weight,
                if part.comparable {
                    ""
                } else {
                    "; no corresponding content"
                }
            );
        }
        for role in &evidence.roles {
            println!(
                "  {}: score {:.4}, weight {:.4}{}",
                role.role_label,
                role.score,
                role.weight,
                if role.comparable {
                    ""
                } else {
                    "; no corresponding content"
                }
            );
        }
    }
    if evidence.representative {
        println!("  Representative content; excerpts do not decompose the average score");
    }
    if evidence.truncated && explain {
        println!("  Explanation is limited; ranking used all selected parts");
    }
    for pair in evidence
        .pairs
        .iter()
        .take(if explain { usize::MAX } else { 1 })
    {
        let seed_label = pair
            .seed
            .as_ref()
            .and_then(|w| w.label.as_deref())
            .unwrap_or("Selected part");
        let candidate_label = pair.candidate.label.as_deref().unwrap_or("Content");
        println!("  Seed ({seed_label}): {}", pair.seed_text);
        if let Some(text) = &pair.candidate.snippet {
            println!("  Match ({candidate_label}): {text}");
        } else {
            println!("  Match: {candidate_label}");
        }
    }
}

fn candidate_mode_label(mode: SimilarityCandidateParts) -> &'static str {
    match mode {
        SimilarityCandidateParts::Any => "any candidate content",
        SimilarityCandidateParts::Corresponding => "corresponding parts",
    }
}

fn core_representation_label(value: atlas_domain::CoreSectionRepresentation) -> &'static str {
    match value {
        atlas_domain::CoreSectionRepresentation::IdentityAndProse => {
            "Name and identity with Core prose"
        }
        atlas_domain::CoreSectionRepresentation::IdentityOnly => "Name and identity only",
        atlas_domain::CoreSectionRepresentation::TraitDefinition => "Trait definition",
        atlas_domain::CoreSectionRepresentation::TraitLabelOnly => "Trait label only",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;

    fn parse(arguments: &[&str]) -> SimilarOptions {
        let Command::Similar(options) = Cli::try_parse_from(arguments).unwrap().command else {
            panic!("similar command expected");
        };
        *options
    }

    #[test]
    fn discovery_is_a_leaf_action_and_defaults_do_not_conflict() {
        let options = parse(&["atlas", "similar", "Brown Mold", "--list-parts", "--json"]);
        assert!(options.list_parts);
        assert!(
            Cli::try_parse_from([
                "atlas",
                "similar",
                "Brown Mold",
                "--list-parts",
                "--page",
                "2"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "atlas",
                "similar",
                "Brown Mold",
                "--list-parts",
                "--method",
                "passages"
            ])
            .is_err()
        );
    }

    #[test]
    fn explicit_parts_require_the_discovery_generation() {
        let unguarded = parse(&["atlas", "similar", "Brown Mold", "--part", "part-123"]);
        let message = recovery::missing_generation(&unguarded).unwrap();
        assert!(message.contains("atlas similar 'Brown Mold' --list-parts"));
        let options = parse(&[
            "atlas",
            "similar",
            "Brown Mold",
            "--part",
            "part-123",
            "--part",
            "part-456",
            "--artifact-generation",
            "generation",
        ]);
        assert_eq!(
            comparison_options(&options)
                .unwrap()
                .unwrap()
                .parts()
                .unwrap()
                .iter()
                .map(|selector| selector.0.as_str())
                .collect::<Vec<_>>(),
            ["part-123", "part-456"]
        );
    }

    #[test]
    fn methods_reject_inapplicable_explicit_options() {
        let average = parse(&[
            "atlas",
            "similar",
            "Brown Mold",
            "--method",
            "content-average",
            "--decay",
            "0.5",
        ]);
        assert!(comparison_options(&average).is_err());
        let baseline = parse(&[
            "atlas",
            "similar",
            "Brown Mold",
            "--method",
            "baseline",
            "--candidate-parts",
            "any",
        ]);
        assert!(comparison_options(&baseline).is_err());
    }

    #[test]
    fn bare_trait_uses_its_default_while_content_flags_select_passages() {
        let seed = SimilaritySeed::Trait {
            trait_key: "trait:fire".parse().unwrap(),
        };
        let bare = parse(&["atlas", "similar", "trait:fire"]);
        let options = comparison_options(&bare)
            .unwrap()
            .unwrap_or_else(|| SimilarityOptions::default_for_seed(&seed));
        assert_eq!(options, SimilarityOptions::Baseline {});
        let explicit = parse(&["atlas", "similar", "trait:fire", "--decay", "0.75"]);
        assert!(matches!(
            comparison_options(&explicit).unwrap(),
            Some(SimilarityOptions::Passages { decay, .. }) if decay == 0.75
        ));
    }

    #[test]
    fn core_source_and_candidate_scope_are_independent_and_discovery_is_strict() {
        let trait_seed = SimilaritySeed::Trait {
            trait_key: "trait:fire".parse().unwrap(),
        };
        let candidate_only = parse(&["atlas", "similar", "trait:fire", "--search-content", "core"]);
        assert_eq!(
            comparison_options(&candidate_only)
                .unwrap()
                .unwrap_or_else(|| SimilarityOptions::default_for_seed(&trait_seed)),
            SimilarityOptions::Baseline {}
        );
        let source = parse(&[
            "atlas",
            "similar",
            "Lich",
            "--sections",
            "core",
            "--search-content",
            "all",
        ]);
        assert!(matches!(
            comparison_options(&source).unwrap(),
            Some(SimilarityOptions::Passages { .. })
        ));
        let baseline = parse(&[
            "atlas",
            "similar",
            "Lich",
            "--method",
            "baseline",
            "--sections",
            "all",
        ]);
        assert!(comparison_options(&baseline).is_err());
        assert!(
            parse(&[
                "atlas",
                "similar",
                "Lich",
                "--list-parts",
                "--sections",
                "core"
            ])
            .list_parts
        );
        for flags in [
            vec!["--sections", "core", "--part", "selector"],
            vec!["--list-parts", "--search-content", "core"],
            vec!["--list-parts", "--decay", "0.5"],
        ] {
            assert!(
                Cli::try_parse_from(["atlas", "similar", "Lich"].into_iter().chain(flags)).is_err()
            );
        }
        assert!(Cli::try_parse_from(["atlas", "search", "--search-content", "core"]).is_err());
        assert!(
            Cli::try_parse_from(["atlas", "search", "fire", "--search-content", "core"]).is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "atlas",
                "search",
                "fire",
                "--search-content",
                "core",
                "--print-filter"
            ])
            .is_err()
        );
    }

    #[test]
    fn decay_preserves_precision_and_rejects_nonfinite_values() {
        let options = parse(&["atlas", "similar", "Brown Mold", "--decay", "0.123456"]);
        assert!(
            matches!(comparison_options(&options).unwrap(), Some(SimilarityOptions::Passages { decay, .. }) if decay == 0.123456)
        );
        for value in ["NaN", "inf", "1.01", "-0.01"] {
            assert!(
                Cli::try_parse_from([
                    "atlas",
                    "similar",
                    "Brown Mold",
                    &format!("--decay={value}")
                ])
                .is_err()
            );
        }
    }
}
