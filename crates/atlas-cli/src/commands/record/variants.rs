use super::{args::VariantsOptions, report_app_error};
use crate::client::{ClientOptions, connect};
use atlas_cli_support::write_json_data;
use std::process::ExitCode;

pub(crate) fn run(o: VariantsOptions) -> Result<ExitCode, String> {
    let s = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(o.path_mode),
        index: o.index,
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, o.json),
    };
    let key = match super::resolve_seed(&s, o.record_ref, o.json)? {
        Ok(key) => key,
        Err(exit) => return Ok(exit),
    };
    let r = match s.variant_group(key) {
        Ok(value) => value,
        Err(error) => return report_app_error(error.into_app_error(), o.json),
    };
    if o.json {
        write_json_data(serde_json::json!({"label":"Suggested variants","availability":r}))?;
    } else {
        use atlas_app_model::RelatedCategoryView;
        match r {
            RelatedCategoryView::Content { content } => {
                println!("Suggested variants for {}", content.seed.title);
                for variant in content.variants {
                    println!("  {} ({})", variant.title, variant.record_key);
                    for evidence in content
                        .evidence
                        .iter()
                        .filter(|e| e.record_key == variant.record_key)
                    {
                        println!(
                            "    {} · {} · {}",
                            evidence.naming_convention, evidence.qualifier, evidence.compatibility
                        );
                    }
                }
                if content.ambiguous {
                    println!("Multiple naming groups are possible.");
                }
                if content.truncated {
                    println!("Additional variants were omitted.");
                }
            }
            RelatedCategoryView::Empty => println!("No suggested variants."),
            RelatedCategoryView::Unavailable { reason } => {
                println!("Suggested variants unavailable: {reason}")
            }
            RelatedCategoryView::Error { error } => return report_app_error(error, false),
        }
    }
    Ok(ExitCode::SUCCESS)
}
