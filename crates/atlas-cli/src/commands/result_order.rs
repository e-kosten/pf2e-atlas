//! Product CLI grammar for complete ordering modes and ordered attribute rules.
use atlas_domain::{ResultSort, SortDirection, SortField, SortRule};

pub(crate) fn requested_sort(
    tokens: &[String],
    seed: Option<u32>,
) -> Result<Option<ResultSort>, String> {
    if tokens.len() == 1 && tokens[0] == "random" {
        return Ok(Some(ResultSort::Random {
            seed: seed.unwrap_or_else(rand::random),
        }));
    }
    if seed.is_some() {
        return Err("--seed requires --sort random as the only sort".into());
    }
    if tokens.is_empty() {
        return Ok(None);
    }
    if tokens.len() == 1 {
        match tokens[0].as_str() {
            "relevance" => return Ok(Some(ResultSort::Relevance {})),
            "saved_order" => return Ok(Some(ResultSort::SavedOrder {})),
            _ => {}
        }
    }
    let rules = tokens
        .iter()
        .map(|token| {
            let (field, direction) = match token.as_str() {
                "name_asc" => (SortField::Name, SortDirection::Asc),
                "name_desc" => (SortField::Name, SortDirection::Desc),
                "level_asc" => (SortField::Level, SortDirection::Asc),
                "level_desc" => (SortField::Level, SortDirection::Desc),
                "unit_price_asc" => (SortField::UnitPrice, SortDirection::Asc),
                "unit_price_desc" => (SortField::UnitPrice, SortDirection::Desc),
                "random" | "relevance" | "saved_order" => {
                    return Err("Random, Relevance and Saved order must be used alone".into());
                }
                _ => return Err(format!("Unknown sort: {token}")),
            };
            Ok(SortRule { field, direction })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sort = ResultSort::Attributes { rules };
    sort.validate().map_err(str::to_owned)?;
    Ok(Some(sort))
}
pub(crate) fn print_random_seed(sort: &ResultSort) {
    if let ResultSort::Random { seed } = sort {
        println!("Random seed: {seed}");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parse(tokens: &[&str], seed: Option<u32>) -> Result<Option<ResultSort>, String> {
        requested_sort(
            &tokens.iter().map(|t| (*t).into()).collect::<Vec<_>>(),
            seed,
        )
    }
    #[test]
    fn ordered_attributes_preserve_priority_and_reject_duplicate_or_mixed_modes() {
        assert_eq!(
            parse(&["level_desc", "unit_price_asc"], None).unwrap(),
            Some(ResultSort::Attributes {
                rules: vec![
                    SortRule {
                        field: SortField::Level,
                        direction: SortDirection::Desc
                    },
                    SortRule {
                        field: SortField::UnitPrice,
                        direction: SortDirection::Asc
                    }
                ]
            })
        );
        for tokens in [
            vec!["name_asc", "name_desc"],
            vec!["relevance", "level_asc"],
            vec!["random", "name_asc"],
            vec!["nonsense"],
        ] {
            assert!(parse(&tokens, None).is_err());
        }
        assert_eq!(parse(&[], None).unwrap(), None);
    }
    #[test]
    fn random_seed_is_explicit_or_generated_only_for_sole_random_mode() {
        for seed in [0, u32::MAX] {
            assert_eq!(
                parse(&["random"], Some(seed)).unwrap(),
                Some(ResultSort::Random { seed })
            );
        }
        assert!(matches!(
            parse(&["random"], None).unwrap(),
            Some(ResultSort::Random { .. })
        ));
        assert!(parse(&[], Some(0)).is_err());
        assert!(parse(&["name_asc"], Some(0)).is_err());
    }
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;
    #[test]
    fn product_sort_flags_preserve_repetition_and_bound_seed_values() {
        let parsed = Cli::try_parse_from([
            "atlas",
            "search",
            "--sort",
            "level_desc",
            "--sort",
            "unit_price_asc",
        ])
        .unwrap();
        let Command::Search(options) = parsed.command else {
            panic!("search command")
        };
        assert_eq!(options.sort, ["level_desc", "unit_price_asc"]);
        assert!(
            matches!(requested_sort(&options.sort,options.seed).unwrap(), Some(ResultSort::Attributes { rules }) if rules.len()==2)
        );
        for seed in ["0", "4294967295"] {
            let parsed = Cli::try_parse_from([
                "atlas",
                "similar",
                "Leather Armor",
                "--sort",
                "random",
                "--seed",
                seed,
            ])
            .unwrap();
            let Command::Similar(options) = parsed.command else {
                panic!("similar command")
            };
            assert_eq!(
                requested_sort(&options.sort, options.seed).unwrap(),
                Some(ResultSort::Random {
                    seed: seed.parse().unwrap()
                })
            );
        }
        for seed in ["-1", "4294967296", "1.5", "bad"] {
            assert!(
                Cli::try_parse_from(["atlas", "search", "--sort", "random", "--seed", seed])
                    .is_err()
            );
        }
        let parsed = Cli::try_parse_from([
            "atlas", "lists", "show", "sample", "--sort", "random", "--seed", "42",
        ])
        .unwrap();
        let Command::Lists(options) = parsed.command else {
            panic!("lists command")
        };
        let crate::commands::lists::args::ListsCommand::Show(options) = options.command else {
            panic!("show command")
        };
        assert_eq!(
            requested_sort(&options.sort, options.seed).unwrap(),
            Some(ResultSort::Random { seed: 42 })
        );
    }
}
