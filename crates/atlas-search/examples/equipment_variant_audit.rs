//! Audit equipment variants and propose grade progressions from a private reference snapshot.

use atlas_domain::{RecordKey, SearchProfileId};
use atlas_index::SqliteIndexReader;
use atlas_search::{
    AtlasRetrievalService, VariantGroupOutcome, VariantGroupRequest, VariantGroupResult,
};
use atlas_storage::Workspace;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (mode, path, schema) = match args.as_slice() {
        [path, schema] => (Mode::Audit, path, schema),
        [option, path, schema] if option == "--propose-grades" => (Mode::ProposeGrades, path, schema),
        _ => return Err("usage: equipment_variant_audit [--propose-grades] <workspace.sqlite> <workspace schema>".into()),
    };
    let session = Workspace::at(path)?.read(schema.parse()?)?;
    let reader = SqliteIndexReader::from_session(session)?;
    let retrieval =
        AtlasRetrievalService::from_index(reader, SearchProfileId::new("bge-base"), None)?;
    let input = std::io::stdin().lock();
    let mut output = BufWriter::new(std::io::stdout().lock());
    match mode {
        Mode::Audit => audit(&retrieval, input, &mut output)?,
        Mode::ProposeGrades => proposals(&retrieval, input, &mut output)?,
    }
    output.flush()?;
    Ok(())
}

fn audit(
    retrieval: &AtlasRetrievalService,
    input: impl BufRead,
    mut output: impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    for key in input.lines() {
        let key = RecordKey::parse(&key?)?;
        let outcome = match retrieval.variant_group(VariantGroupRequest { record_key: &key })? {
            VariantGroupOutcome::Content(group) => json!({
                "kind": "content",
                "family": group.base_name,
                "members": group.variants.iter().map(|member| &member.key).collect::<Vec<_>>(),
                "verified": group.verified,
                "qualifiers": group.evidence.iter().map(|e| &e.qualifier).collect::<Vec<_>>(),
                "ambiguous": group.ambiguous,
                "truncated": group.truncated,
            }),
            VariantGroupOutcome::Missing => json!({ "kind": "missing" }),
            VariantGroupOutcome::Empty => json!({ "kind": "empty" }),
            VariantGroupOutcome::Unavailable { reason } => {
                json!({ "kind": "unavailable", "reason": reason })
            }
        };
        serde_json::to_writer(&mut output, &json!({ "key": key, "outcome": outcome }))?;
        writeln!(output)?;
    }
    Ok(())
}

enum Mode {
    Audit,
    ProposeGrades,
}

struct GradeProposal {
    report: serde_json::Value,
    agreeing_seeds: BTreeSet<RecordKey>,
    inconsistent: bool,
}

fn propose_grade_group(group: &VariantGroupResult) -> Option<(Vec<RecordKey>, serde_json::Value)> {
    if group.verified || group.ambiguous || group.truncated {
        return None;
    }
    let publication = group.seed.publication_title.as_value()?;
    let remaster = group.seed.publication_remaster.as_value()?;
    let item_type = group.seed.source_type.as_value()?;
    let mut members = Vec::with_capacity(group.variants.len());
    for (member, evidence) in group.variants.iter().zip(&group.evidence) {
        if member.key != evidence.record
            || member.document_kind != "Item"
            || member.key.pack().as_str() != "equipment-srd"
            || member.source_type.as_value() != Some(item_type)
            || member.publication_title.as_value() != Some(publication)
            || member.publication_remaster.as_value() != Some(remaster)
        {
            return None;
        }
        let grade = match evidence.qualifier.as_str() {
            "Minor" => 0,
            "Lesser" => 1,
            "Moderate" => 2,
            "Greater" => 3,
            "Major" => 4,
            "True" | "Supreme" => 5,
            _ => return None,
        };
        let level = member.level.as_value()?.as_u64()?;
        members.push((grade, level, member, &evidence.qualifier));
    }
    members.sort_by_key(|(grade, _, _, _)| *grade);
    if members.len() != group.variants.len()
        || members.len() < 2
        || members
            .windows(2)
            .any(|pair| pair[0].0 >= pair[1].0 || pair[0].1 >= pair[1].1)
    {
        return None;
    }
    let mut keys: Vec<_> = members
        .iter()
        .map(|(_, _, member, _)| member.key.clone())
        .collect();
    keys.sort();
    let report = json!({
        "family": group.base_name,
        "item_type": item_type,
        "publication": publication,
        "remaster": remaster,
        "members": members.iter().map(|(_, level, member, grade)| json!({
            "key": member.key,
            "name": member.name.as_value(),
            "grade": grade,
            "level": level,
            "source_path": member.source_path,
        })).collect::<Vec<_>>(),
    });
    Some((keys, report))
}

fn proposals(
    retrieval: &AtlasRetrievalService,
    input: impl BufRead,
    mut output: impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut candidates: BTreeMap<Vec<RecordKey>, GradeProposal> = BTreeMap::new();
    for key in input.lines() {
        let key = RecordKey::parse(&key?)?;
        let VariantGroupOutcome::Content(group) =
            retrieval.variant_group(VariantGroupRequest { record_key: &key })?
        else {
            continue;
        };
        let Some((members, report)) = propose_grade_group(&group) else {
            continue;
        };
        let proposal = candidates.entry(members).or_insert_with(|| GradeProposal {
            report: report.clone(),
            agreeing_seeds: BTreeSet::new(),
            inconsistent: false,
        });
        proposal.inconsistent |= proposal.report != report;
        proposal.agreeing_seeds.insert(key);
    }
    for (members, proposal) in candidates {
        if proposal.inconsistent || !proposal.agreeing_seeds.into_iter().eq(members) {
            continue;
        }
        serde_json::to_writer(&mut output, &proposal.report)?;
        writeln!(output)?;
    }
    Ok(())
}
