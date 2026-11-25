use super::args::SearchOptions;
use crate::commands::{
    followup::{
        EvidenceDetail, OccurrenceArgs, filter_args, open_record, shell_command, workspace_args,
    },
    graph::evidence,
    product,
    record::navigation::Locator,
};
use atlas_app_model::{
    RecordNavigationView, RetrievalModeView, SearchModeSummary, SearchResultRow, SearchResultsPage,
};
use atlas_domain::{ResultSort, SearchContentScope, SearchRanking};
use serde_json::{Value, json};

fn selected(row: &SearchResultRow) -> RecordNavigationView {
    if let Some(witness) = row.best_match() {
        return witness.navigation.clone();
    }
    for edge in row
        .relationship_evidence
        .iter()
        .flat_map(|e| &e.occurrences)
    {
        if edge.source_navigation.record_key == row.record.record_key {
            return edge.source_navigation.clone();
        }
        if let Some(target) = edge.destination.record_target()
            && target.record_key == row.record.record_key
        {
            return target.clone();
        }
    }
    crate::commands::record::navigation::root(&row.record.record_key)
}

fn next_page(
    o: &SearchOptions,
    result: &SearchResultsPage,
    workspace: &[String],
    filters: &[String],
) -> Result<Option<Vec<String>>, String> {
    let Some(next) = result.page.next_page else {
        return Ok(None);
    };
    let mut args = vec![
        "atlas".into(),
        "search".into(),
        format!("--page={next}"),
        format!("--limit={}", result.page.size),
        format!("--artifact-generation={}", result.artifact_generation),
    ];
    args.extend_from_slice(workspace);
    args.extend_from_slice(filters);
    for value in &o.sort {
        args.push(format!("--sort={value}"));
    }
    if let ResultSort::Random { seed } = result.sort {
        args.push(format!("--seed={seed}"));
    }
    if o.json {
        args.push("--json".into());
    }
    if o.detail == EvidenceDetail::Full {
        args.push("--detail=full".into());
    }
    if let Some(path) = &o.embedding_cache_path {
        args.push(format!(
            "--embedding-cache-path={}",
            path.to_str().ok_or(
                "Cannot encode the embedding cache path in a command argument for display."
            )?
        ));
    }
    if let SearchModeSummary::TextSearch { query, mode } = &result.mode {
        let retrieval = match mode {
            RetrievalModeView::Lexical => "fts",
            RetrievalModeView::Semantic => "vector",
            RetrievalModeView::Hybrid => "hybrid",
        };
        args.push(format!("--retrieval={retrieval}"));
        if matches!(result.search_content, SearchContentScope::Core) {
            args.push("--search-content=core".into());
        }
        if matches!(result.ranking, SearchRanking::Rrf) {
            args.push("--ranking=rrf".into());
        }
        if *mode != RetrievalModeView::Lexical || o.search_profile.is_some() {
            args.push(format!("--search-profile={}", result.search_profile));
        }
        args.push("--".into());
        args.push(query.clone());
    } else if let Some(profile) = &o.search_profile {
        args.push(format!("--search-profile={profile}"));
    }
    Ok(Some(args))
}

fn source_owner<'a>(
    row: &'a SearchResultRow,
    result: &'a SearchResultsPage,
    clause_id: &str,
    edge: &atlas_app_model::RecordRelationshipView,
) -> Option<&'a atlas_app_model::RecordSummaryView> {
    if row.record.record_key == edge.source_navigation.record_key {
        return Some(&row.record);
    }
    result
        .relationship_clauses
        .iter()
        .find(|c| {
            c.clause_id == clause_id && c.record.record_key == edge.source_navigation.record_key
        })
        .map(|c| &c.record)
}

fn compact_row(result: &SearchResultsPage, row: &SearchResultRow) -> Value {
    let evidence: Vec<_> = row
        .relationship_evidence
        .iter()
        .map(|proof| {
            let occurrences: Vec<_> = proof
                .occurrences
                .iter()
                .map(|edge| {
                    evidence::occurrence(
                        edge,
                        source_owner(row, result, &proof.clause_id, edge),
                        &result.artifact_generation,
                    )
                })
                .collect();
            json!({"clause_id":proof.clause_id, "page":proof.page, "occurrences":occurrences})
        })
        .collect();
    let mut summary =
        json!({"record":evidence::identity(&row.record), "relationship_evidence":evidence});
    if let Some(witness) = row.best_match() {
        summary["best_match"] =
            json!({"label":witness.label,"preview":witness.preview,"lane":witness.lane});
    }
    summary
}

fn json_page(result: &SearchResultsPage, continuation: &OccurrenceArgs<'_>) -> Value {
    let mut data = if continuation.detail == EvidenceDetail::Full {
        json!(result)
    } else {
        json!({
            "result_detail":result.result_detail, "artifact_generation":result.artifact_generation,"mode":result.mode,"search_profile":result.search_profile,
            "coverage":result.coverage,"sort":result.sort,"search_content":result.search_content,"ranking":result.ranking,
            "page":result.page,"relationship_clauses":result.relationship_clauses.iter().map(|c| json!({
                "clause_id":c.clause_id,"title":c.title,"relationship":c.relationship
            })).collect::<Vec<_>>(), "rows":result.rows.iter().map(|row| compact_row(result,row)).collect::<Vec<_>>()
        })
    };
    if let Some(recipe) = &result.core_recipe {
        data["core_recipe"] = json!(recipe);
    }
    for (index, row) in result.rows.iter().enumerate() {
        let entry = &mut data["rows"][index];
        let nav = selected(row);
        entry["navigation"] = Locator::new(continuation.generation, &nav).compact();
        if let Some(workspace) = continuation.workspace {
            entry["open_args"] = json!(open_record(&nav, continuation.generation, workspace, true));
        }
        for (i, proof) in row.relationship_evidence.iter().enumerate() {
            if let Some(next) =
                continuation.next(&row.record.record_key, &proof.clause_id, &proof.page)
            {
                entry["relationship_evidence"][i]["next_args"] = json!(next);
            }
        }
    }
    data
}

fn print_page(result: &SearchResultsPage, continuation: &OccurrenceArgs<'_>) {
    super::super::result_order::print_random_seed(&result.sort);
    for row in &result.rows {
        product::print_summary(&row.record);
        if let Some(publication) = &row.record.publication {
            println!("  {publication}");
        }
        let witnesses: Vec<_> = if continuation.detail == EvidenceDetail::Full {
            row.matches.iter().collect()
        } else {
            row.best_match().into_iter().collect()
        };
        for witness in witnesses {
            if let Some(label) = &witness.label
                && label != &row.record.title
            {
                println!("  Matched: {label}");
            }
            if let Some(preview) = &witness.preview {
                println!("  {}", product::match_preview_text(preview));
            }
        }
        for proof in &row.relationship_evidence {
            for edge in &proof.occurrences {
                println!(
                    "  {}",
                    evidence::line(edge, source_owner(row, result, &proof.clause_id, edge))
                );
            }
            if let Some(args) =
                continuation.next(&row.record.record_key, &proof.clause_id, &proof.page)
            {
                println!("  More references: {}", shell_command(&args));
            }
        }
        if let Some(workspace) = continuation.workspace {
            let open = open_record(&selected(row), continuation.generation, workspace, false);
            println!("  {}", shell_command(&open));
        }
        println!();
    }
    product::print_search_page_count(result);
}

pub(super) fn write(o: &SearchOptions, result: &SearchResultsPage) -> Result<(), String> {
    let workspace = workspace_args(o.path_mode, o.index.as_deref());
    let filters = filter_args(&o.filter_options);
    let continuation = OccurrenceArgs {
        workspace: workspace.as_deref().ok(),
        filter: &filters,
        generation: &result.artifact_generation,
        json: o.json,
        detail: o.detail,
    };
    let next = workspace
        .as_deref()
        .map_err(Clone::clone)
        .and_then(|args| next_page(o, result, args, &filters));
    if o.json {
        let mut data = json_page(result, &continuation);
        match next {
            Ok(Some(args)) => data["next_args"] = json!(args),
            Err(reason) => data["followup_unavailable"] = json!(reason),
            Ok(None) => {}
        }
        atlas_cli_support::write_json_data(data)
    } else {
        print_page(result, &continuation);
        match next {
            Ok(Some(args)) => println!("Next page: {}", shell_command(&args)),
            Err(reason) => println!("{reason}"),
            Ok(None) => {}
        }
        Ok(())
    }
}
