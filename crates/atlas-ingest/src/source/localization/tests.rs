use super::*;
use atlas_record::source_content::{
    ContentAudience, ContentDiagnosticCode, ContentVisibilityRule, LocalizationResolver,
    SourceContentLocator, prepare_source_content,
};
use serde_json::json;

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("static")).unwrap();
    fs::create_dir(root.path().join("static/lang")).unwrap();
    fs::write(
        root.path().join("module.json"),
        json!({"languages":[
            {"lang":"en","path":"lang/en.json"},
            {"lang":"en","path":"lang/action-en.json"},
            {"lang":"en","path":"lang/re-en.json"},
            {"lang":"fr","path":"lang/fr.json"}
        ]})
        .to_string(),
    )
    .unwrap();
    for (name, values) in [
        (
            "en",
            json!({"Shared":"Base", "OnlyEnglish":"English fallback"}),
        ),
        (
            "action-en",
            json!({"Action":"<p>@Localize[Rule] @Check[flat|dc:5]</p>"}),
        ),
        (
            "re-en",
            json!({"Rule":"Complete English rules.","Shared":"Later declaration"}),
        ),
        ("fr", json!({"Shared":"French"})),
        ("sf2e-overrides-en", json!({"Shared":"Undeclared override"})),
    ] {
        fs::write(
            root.path().join(format!("static/lang/{name}.json")),
            values.to_string(),
        )
        .unwrap();
    }
    root
}

#[test]
fn declared_english_catalogs_expand_nested_content_in_manifest_order() {
    let root = fixture();
    let catalog = LocalizationCatalog::load(root.path(), "en", None).unwrap();
    assert_eq!(catalog.localized_value("Shared"), Some("Later declaration"));
    let content = prepare_source_content(
        SourceContentLocator {
            record: "items:aaaaaaaaaaaaaaaa".parse().unwrap(),
            owners: vec![],
            field: "/system/description/value".into(),
        },
        "@Localize[Action]",
        ContentAudience {
            include_gm: true,
            include_owner: true,
            implicit_check_dc: ContentVisibilityRule::Gm,
        },
        ContentVisibilityRule::All,
        Some(&catalog),
        None,
        &atlas_record::source_content::FormulaContext::default(),
    )
    .unwrap();
    assert!(content.text.contains("Complete English rules."));
    assert!(!content.html.contains("@Localize"));
    assert_eq!(content.interactions.len(), 1);
    assert!(content.diagnostics.is_empty());
}

#[test]
fn composed_english_remains_the_selected_locale_fallback() {
    let root = fixture();
    let catalog = LocalizationCatalog::load(root.path(), "fr", None).unwrap();
    assert_eq!(catalog.localized_value("Shared"), Some("French"));
    assert_eq!(
        catalog.localized_value("Rule"),
        Some("Complete English rules.")
    );
    assert_eq!(
        catalog.localized_value("OnlyEnglish"),
        Some("English fallback")
    );
}

#[test]
fn catalog_identity_binds_order_paths_and_all_bytes() {
    let root = fixture();
    let hash = || {
        LocalizationCatalog::load(root.path(), "en", None)
            .unwrap()
            .english_sha256
    };
    let original = hash();
    let rules = root.path().join("static/lang/re-en.json");
    let bytes = fs::read(&rules).unwrap();
    fs::write(
        &rules,
        json!({"Rule":"Changed rules.","Shared":"Later declaration"}).to_string(),
    )
    .unwrap();
    assert_ne!(original, hash());
    fs::write(&rules, &bytes).unwrap();
    assert_eq!(original, hash());
    let manifest_path = root.path().join("module.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["languages"].as_array_mut().unwrap().swap(0, 2);
    fs::write(&manifest_path, manifest.to_string()).unwrap();
    let reordered = hash();
    assert_ne!(original, reordered);
    fs::rename(&rules, root.path().join("static/lang/renamed.json")).unwrap();
    manifest["languages"][0]["path"] = json!("lang/renamed.json");
    fs::write(&manifest_path, manifest.to_string()).unwrap();
    assert_ne!(reordered, hash());
}

#[test]
fn explicit_manifest_uses_its_catalogs_in_source_capture() {
    let root = fixture();
    fs::create_dir(root.path().join("translations")).unwrap();
    fs::write(
        root.path().join("translations/english.json"),
        json!({"Explicit":"Chosen manifest"}).to_string(),
    )
    .unwrap();
    let manifest = root.path().join("custom.json");
    fs::write(
        &manifest,
        json!({"languages":[{"lang":"en","path":"translations/english.json"}]}).to_string(),
    )
    .unwrap();
    let capture =
        crate::source_fingerprint::SourceCapture::capture(root.path(), Some(&manifest)).unwrap();
    let catalog =
        LocalizationCatalog::load_captured(root.path(), "en", Some(&manifest), Some(&capture))
            .unwrap();
    assert_eq!(catalog.localized_value("Explicit"), Some("Chosen manifest"));
    assert_eq!(catalog.localized_value("Shared"), None);
    let before = capture.fingerprint;
    fs::write(
        root.path().join("translations/english.json"),
        json!({"Explicit":"Changed"}).to_string(),
    )
    .unwrap();
    assert_ne!(
        before,
        crate::compute_source_fingerprint(root.path(), Some(&manifest)).unwrap()
    );
}

#[test]
fn missing_invalid_or_changed_declared_catalogs_fail_instead_of_partial_loading() {
    let root = fixture();
    let rules = root.path().join("static/lang/re-en.json");
    let capture = crate::source_fingerprint::SourceCapture::capture(root.path(), None).unwrap();
    let bytes = fs::read(&rules).unwrap();
    fs::write(&rules, "{}").unwrap();
    assert!(LocalizationCatalog::load_captured(root.path(), "en", None, Some(&capture)).is_err());
    fs::write(&rules, "[]").unwrap();
    assert!(LocalizationCatalog::load(root.path(), "en", None).is_err());
    fs::remove_file(&rules).unwrap();
    assert!(
        LocalizationCatalog::load(root.path(), "en", None)
            .unwrap_err()
            .to_string()
            .contains("re-en.json")
    );
    fs::write(&rules, bytes).unwrap();
    for invalid in ["../outside.json", "/outside.json", ""] {
        assert!(resolve_catalog_path(root.path(), invalid).is_err());
    }
}

#[test]
#[ignore = "requires the pinned PF2e source; set ATLAS_PF2E_SOURCE and run explicitly"]
fn pinned_english_localize_references_have_resolvable_content() {
    let root = PathBuf::from(std::env::var_os("ATLAS_PF2E_SOURCE").expect("ATLAS_PF2E_SOURCE"));
    let catalog = LocalizationCatalog::load(&root, "en", None).unwrap();
    let keys: Vec<String> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/english-localize-keys.json"
    ))
    .unwrap();
    for key in keys {
        let content = prepare_source_content(
            SourceContentLocator {
                record: "items:aaaaaaaaaaaaaaaa".parse().unwrap(),
                owners: vec![],
                field: "/system/description/value".into(),
            },
            &format!("@Localize[{key}]"),
            ContentAudience {
                include_gm: true,
                include_owner: true,
                implicit_check_dc: ContentVisibilityRule::Gm,
            },
            ContentVisibilityRule::All,
            Some(&catalog),
            None,
            &atlas_record::source_content::FormulaContext::default(),
        )
        .unwrap();
        assert!(!content.text.trim().is_empty(), "{key}");
        assert!(
            !content
                .diagnostics
                .iter()
                .any(|d| d.code == ContentDiagnosticCode::UnresolvedLocalization),
            "{key}: {:?}",
            content.diagnostics
        );
    }
}
