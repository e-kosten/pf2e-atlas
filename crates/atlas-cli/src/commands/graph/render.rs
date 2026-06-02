use std::collections::BTreeMap;

use super::data::{
    GraphLinksData, GraphRemasterData, GraphSectionJson, GraphUsesData, GraphVariantsData,
};

pub(super) fn print_graph_links(
    data: &GraphLinksData,
    outgoing_limit: usize,
    backlink_limit: usize,
) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.kind
    );
    print_section("Outgoing", &data.outgoing, outgoing_limit, true);
    print_section("Backlinks", &data.backlinks, backlink_limit, false);
}

pub(super) fn print_graph_uses(data: &GraphUsesData, limit: usize) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.kind
    );
    print_section("Uses", &data.uses, limit, false);
}

pub(super) fn print_graph_variants(data: &GraphVariantsData) {
    if let Some(seed) = &data.seed {
        println!(
            "{}\t{}\t{}",
            seed.record.key, seed.record.name, seed.record.kind
        );
    }
    let Some(group_key) = &data.variant_group_key else {
        println!("Variants: none");
        return;
    };
    println!("Variant group: {group_key}");
    if data.variants.is_empty() {
        println!("Variants: none");
        return;
    }
    for variant in &data.variants {
        let current = if variant.is_seed { " *" } else { "" };
        let label = variant
            .variant_label
            .as_deref()
            .filter(|label| !label.is_empty())
            .unwrap_or("-");
        let axes = if variant.variant_axes.is_empty() {
            "-".to_string()
        } else {
            variant.variant_axes.join(", ")
        };
        println!(
            "-{} {}\t{}\t{}\tlabel={label}\taxes={axes}",
            current, variant.record.key, variant.record.name, variant.record.kind
        );
    }
}

pub(super) fn print_graph_remaster(data: &GraphRemasterData) {
    println!(
        "{}\t{}\t{}",
        data.seed.record.key, data.seed.record.name, data.seed.record.kind
    );
    if data.links.is_empty() {
        println!("Remaster links: none");
        return;
    }
    for link in &data.links {
        println!(
            "- {}\t{} -> {}\t{} -> {}",
            link.direction,
            link.legacy.key,
            link.remaster.key,
            link.legacy.name,
            link.remaster.name
        );
    }
}

fn print_section(label: &str, section: &GraphSectionJson, limit: usize, outgoing: bool) {
    if limit == 0 {
        println!("{label}: disabled");
        return;
    }
    println!(
        "{label}: {} records, {} edges (of {} records, {} edges)",
        section.records.len(),
        section.edges.len(),
        section.total_records,
        section.total_edges
    );
    let names_by_key = section
        .records
        .iter()
        .map(|record| (record.key.clone(), record.name.clone()))
        .collect::<BTreeMap<_, _>>();
    for edge in &section.edges {
        let neighbor_key = if outgoing { &edge.to } else { &edge.from };
        let label = edge
            .display_text
            .as_deref()
            .or_else(|| names_by_key.get(neighbor_key).map(String::as_str))
            .unwrap_or(neighbor_key);
        if outgoing {
            println!("- {label} -> {neighbor_key}");
        } else {
            println!("- {neighbor_key} -> {label}");
        }
    }
}
