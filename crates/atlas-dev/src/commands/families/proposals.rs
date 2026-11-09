use atlas_domain::RecordKey;
use atlas_search::{
    AtlasRetrievalService, VariantGroupOutcome, VariantGroupRequest, VariantGroupResult,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use super::EQUIPMENT_PACK;

#[derive(Clone, PartialEq, Eq, Serialize)]
pub(super) struct Proposal {
    pub(super) family: String,
    item_type: String,
    publication: String,
    remaster: bool,
    pub(super) members: Vec<ProposalMember>,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub(super) struct ProposalMember {
    key: RecordKey,
    name: Option<String>,
    grade: String,
    level: u64,
    source_path: String,
}

struct GradeProposal {
    report: Proposal,
    agreeing_seeds: BTreeSet<RecordKey>,
    inconsistent: bool,
}

fn propose_grade_group(group: &VariantGroupResult) -> Option<(Vec<RecordKey>, Proposal)> {
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
            || member.key.pack().as_str() != EQUIPMENT_PACK
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
    let report = Proposal {
        family: group.base_name.clone(),
        item_type: item_type.clone(),
        publication: publication.clone(),
        remaster: *remaster,
        members: members
            .into_iter()
            .map(|(_, level, member, grade)| ProposalMember {
                key: member.key.clone(),
                name: member.name.as_value().cloned(),
                grade: grade.clone(),
                level,
                source_path: member.source_path.clone(),
            })
            .collect(),
    };
    Some((keys, report))
}

pub(super) fn proposals(
    retrieval: &AtlasRetrievalService,
    keys: &[RecordKey],
) -> Result<Vec<Proposal>, String> {
    let mut candidates: BTreeMap<Vec<RecordKey>, GradeProposal> = BTreeMap::new();
    for key in keys {
        let VariantGroupOutcome::Content(group) = retrieval
            .variant_group(VariantGroupRequest { record_key: key })
            .map_err(|error| error.to_string())?
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
        proposal.agreeing_seeds.insert(key.clone());
    }
    Ok(candidates
        .into_iter()
        .filter(|(members, proposal)| {
            !proposal.inconsistent && proposal.agreeing_seeds.iter().eq(members)
        })
        .map(|(_, proposal)| proposal.report)
        .collect())
}
