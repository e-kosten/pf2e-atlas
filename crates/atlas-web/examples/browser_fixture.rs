//! Model-free responsive browser fixture over the real checked artifact and web app.
//!
//! Run `browser_fixture [127.0.0.1:8794] [fresh-run-directory]`. The optional
//! directory is retained for inspection; otherwise a checkout-local temporary
//! directory isolates reference and mutable state for this process.
//! Stored vectors are structural test data, not model output or ranking evidence.

#[path = "browser_fixture/corpus.rs"]
mod corpus;
#[path = "browser_fixture/reference_data.rs"]
mod reference_data;

use std::{error::Error, fs, io, net::SocketAddr, path::PathBuf};

use atlas_app_model::AppReadinessStatus;
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_domain::{SearchProfileId, TraitKey};
use atlas_index::TraitDefinition;
use atlas_runtime::AtlasPathMode;
use atlas_search::test_support::input;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let address: SocketAddr = args.next().as_deref().unwrap_or("127.0.0.1:8794").parse()?;
    if !address.ip().is_loopback() || args.len() > 1 {
        return Err(io::Error::other(
            "usage: browser_fixture [loopback-address:port] [fresh-run-directory]",
        )
        .into());
    }
    let listener = TcpListener::bind(address).await?;
    let (run_directory, _temporary_directory) = match args.next() {
        Some(path) => {
            let path = PathBuf::from(path);
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                fs::create_dir_all(parent)?;
            }
            fs::create_dir(&path)?;
            (path, None)
        }
        None => {
            let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../scratch/responsive-validation/runs");
            fs::create_dir_all(&parent)?;
            let directory = tempfile::Builder::new()
                .prefix("browser-fixture-")
                .tempdir_in(parent)?;
            (directory.path().to_path_buf(), Some(directory))
        }
    };
    let index_path = run_directory.join("artifact.sqlite");
    let mut records = corpus::records();
    records.push(reference_data::darkvision_record());
    let mut fixture = input(records, true);
    fixture.remaster_pairs.push(atlas_index::SourceRemasterPairInput {
        legacy: "browser-editions:legacy0000000001".parse()?,
        remaster: "browser-editions:remaster00000001".parse()?,
        provenance: Some(serde_json::from_value(serde_json::json!({"record":"browser-journals:editionjournal01","owners":[{"collection":"/pages","identity":{"Stable":"editionpage00001"}}],"field":"/text/content"}))?),
    });
    // Explicit short synthetic definitions and seeds keep the normal trait reading
    // and Similar paths available without claiming model inference or ranking quality.
    for (token, label, markup) in [
        (
            "light",
            "Light",
            "<p>Fixture light trait marks bright lantern effects.</p>",
        ),
        (
            "agile",
            "Agile",
            "<p>Fixture agile trait explains nimble follow-up attacks.</p>",
        ),
    ] {
        let key = TraitKey::new(token)?;
        let prepared = atlas_record::source_content::prepare_reference_definition(
            atlas_domain::DefinitionKey::from(key.clone()),
            markup,
            fixture.context.audience,
            None,
            None,
        )?;
        fixture.traits.push(TraitDefinition {
            key: key.clone(),
            label: label.into(),
            label_key: None,
            description_key: None,
            description_html: Some(prepared.html.clone()),
            authored_markup: Some(markup.into()),
            prepared: Some(prepared),
            catalogs: vec![],
            source_locations: vec![],
            source_revision: None,
            indexing_locale: "en".into(),
            content_locale: Some("en".into()),
        });
    }
    // Give distinct roots positive overlap so the default passage comparison
    // exercises result navigation. The shared fixture's orthogonal vectors
    // correctly produce no positive passage matches. These remain structural
    // vectors, not model output or evidence of semantic ranking quality.
    for unit in &mut fixture.semantic_units {
        assert_eq!(unit.vector.last(), Some(&0.0));
        for component in &mut unit.vector {
            *component *= std::f32::consts::FRAC_1_SQRT_2;
        }
        if let Some(common) = unit.vector.last_mut() {
            *common = std::f32::consts::FRAC_1_SQRT_2;
        }
    }
    let glossary = reference_data::attach(&mut fixture)?;
    let workspace = atlas_runtime::initialize_workspace(&index_path)?;
    atlas_index::test_support::publish_input_in_with_references(
        &workspace,
        atlas_runtime::WORKSPACE_SCHEMA_VERSION,
        &fixture,
        &glossary.entries,
        &glossary.notices,
    )?;
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        default_search_profile: Some(SearchProfileId::new("bge-base")),
        path_mode: AtlasPathMode::Global,
        source_root: Some(run_directory.join("unused-source")),
        embedding_cache_root: Some(run_directory.join("unused-models")),
        index_path: Some(index_path),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoQueryInference,
    })?;
    let readiness = service.readiness();
    if readiness.status != AppReadinessStatus::Ready
        || readiness.search_profiles.len() != 1
        || !readiness.search_profiles[0].stored_vectors
        || readiness.search_profiles[0].assets_ready
        || readiness.search_profiles[0].usable
    {
        return Err(io::Error::other(format!(
            "unexpected structural fixture readiness: {readiness:?}"
        ))
        .into());
    }
    println!(
        "Atlas browser fixture listening at http://{} (state: {}; structural vectors, query models unavailable)",
        listener.local_addr()?,
        run_directory.display()
    );
    axum::serve(listener, atlas_web::router(service))
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("failed to listen for fixture shutdown: {error}");
            }
        })
        .await?;
    Ok(())
}
