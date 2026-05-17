use std::collections::BTreeMap;

use crate::generated::afflictions::AfflictionOccurrence;
use crate::records::NormalizedRecord;

pub(super) fn cluster_affliction_occurrences(
    occurrences: Vec<AfflictionOccurrence>,
) -> Vec<Vec<AfflictionOccurrence>> {
    if occurrences.len() <= 1 {
        return if occurrences.is_empty() {
            Vec::new()
        } else {
            vec![occurrences]
        };
    }

    let mut parent = (0..occurrences.len()).collect::<Vec<_>>();
    for left in 0..occurrences.len() {
        for right in (left + 1)..occurrences.len() {
            if occurrences[left]
                .candidate_keys
                .iter()
                .any(|key| occurrences[right].candidate_keys.contains(key))
            {
                union_parent(&mut parent, left, right);
            }
        }
    }
    let mut clusters = BTreeMap::<usize, Vec<AfflictionOccurrence>>::new();
    for (index, occurrence) in occurrences.into_iter().enumerate() {
        let root = find_parent(&mut parent, index);
        clusters.entry(root).or_default().push(occurrence);
    }
    clusters.into_values().collect()
}

pub(super) fn choose_affliction_authoritative_candidate(
    occurrences: &[AfflictionOccurrence],
) -> (&AfflictionOccurrence, Option<&NormalizedRecord>) {
    let representative = occurrences
        .iter()
        .min_by(|left, right| {
            right
                .source_record
                .is_some()
                .cmp(&left.source_record.is_some())
                .then_with(|| {
                    left.host_record
                        .key
                        .to_string()
                        .cmp(&right.host_record.key.to_string())
                })
                .then_with(|| left.occurrence_ref.cmp(&right.occurrence_ref))
        })
        .expect("non-empty cluster");
    (representative, representative.source_record.as_ref())
}

fn find_parent(parent: &mut [usize], index: usize) -> usize {
    if parent[index] != index {
        let root = find_parent(parent, parent[index]);
        parent[index] = root;
    }
    parent[index]
}

fn union_parent(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find_parent(parent, left);
    let right_root = find_parent(parent, right);
    if left_root != right_root {
        parent[right_root] = left_root;
    }
}
