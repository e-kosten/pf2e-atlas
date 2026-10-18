use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Group {
    label: String,
    formula: String,
}
#[derive(Clone, Debug, Deserialize)]
struct Request {
    id: String,
    groups: Vec<Group>,
    #[serde(default)]
    delay_ms: u64,
    #[serde(default)]
    fail: bool,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
struct Die {
    sides: u64,
    value: u64,
    kept: bool,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
struct GroupResult {
    label: String,
    formula: String,
    total: i64,
    dice: Vec<Die>,
    modifier: i64,
    breakdown: String,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
struct Response {
    id: String,
    groups: Vec<GroupResult>,
    total: i64,
}
struct Cached {
    id: String,
    groups: Vec<Group>,
    response: Response,
}
#[derive(Default)]
struct Cache(VecDeque<Cached>);
type Shared = Arc<Mutex<Cache>>;
type Error = (StatusCode, Json<serde_json::Value>);
fn error(status: StatusCode, message: &str) -> Error {
    (status, Json(serde_json::json!({"error":message})))
}

// Explicit prototype vocabulary: this is an allowlist, not an expression grammar.
// The evaluator owns arithmetic and random outcomes; these facts annotate its history.
const PRESETS: &[(&str, &[u64])] = &[
    ("1d20", &[20]),
    ("2d6", &[6, 6]),
    ("8d6", &[6, 6, 6, 6, 6, 6, 6, 6]),
    ("1d8+1d6", &[8, 6]),
    ("2d20K1", &[20, 20]),
    ("2d20k1", &[20, 20]),
    ("1d8", &[8]),
    ("1d6", &[6]),
    ("1d4", &[4]),
    ("1d12", &[12]),
    ("1d100", &[100]),
    ("1d10", &[10]),
];
fn preset(formula: &str) -> Result<(&'static str, &'static [u64], i64), String> {
    if formula.len() > 40 {
        return Err("Formula too long".into());
    }
    let compact: String = formula.chars().filter(|c| !c.is_whitespace()).collect();
    for &(base, sides) in PRESETS {
        if compact == base {
            return Ok((base, sides, 0));
        }
        for modifier in -100i64..=100 {
            if compact == format!("{base}{modifier:+}") {
                return Ok((base, sides, modifier));
            }
        }
    }
    Err("Use a prototype preset with a modifier between -100 and 100".into())
}
fn evaluate(group: &Group, rng: &mut impl rand::Rng) -> Result<GroupResult, String> {
    let (base, sides, modifier) = preset(&group.formula)?;
    let formula = format!("{base}{modifier:+}");
    let result = caith::Roller::new(&formula)
        .map_err(|e| e.to_string())?
        .roll_with(rng)
        .map_err(|e| e.to_string())?;
    let roll = result.as_single().ok_or("Expected a single expression")?;
    let faces: Vec<u64> = roll
        .get_history()
        .iter()
        .filter_map(|h| match h {
            caith::RollHistory::Roll(dice) => Some(dice.iter().map(|d| d.res)),
            _ => None,
        })
        .flatten()
        .collect();
    if faces.len() != sides.len() {
        return Err("Evaluator returned unexpected dice history".into());
    }
    let total = roll.get_total();
    let kept_value = total - modifier;
    let is_keep = matches!(base, "2d20K1" | "2d20k1");
    let mut selected = false;
    let dice = faces
        .iter()
        .zip(sides)
        .map(|(&value, &sides)| {
            let kept = !is_keep || (!selected && value as i64 == kept_value);
            if is_keep && kept {
                selected = true;
            }
            Die { sides, value, kept }
        })
        .collect::<Vec<_>>();
    if is_keep && !selected {
        return Err("Evaluator total does not identify a kept die".into());
    }
    let breakdown = format_breakdown(&dice, modifier, total);
    Ok(GroupResult {
        label: group.label.clone(),
        formula: group.formula.clone(),
        total,
        dice,
        modifier,
        breakdown,
    })
}

fn format_breakdown(dice: &[Die], modifier: i64, total: i64) -> String {
    let faces = dice
        .iter()
        .map(|die| {
            let discarded = if die.kept { "" } else { " (discarded)" };
            format!("d{}: {}{discarded}", die.sides, die.value)
        })
        .collect::<Vec<_>>()
        .join(", ");
    if modifier == 0 {
        format!("{faces}; total {total}")
    } else {
        format!("{faces}; modifier {modifier:+}; total {total}")
    }
}
impl Cache {
    fn roll(&mut self, request: &Request) -> Result<Response, Error> {
        if let Some(existing) = self.0.iter().find(|c| c.id == request.id) {
            if existing.groups != request.groups {
                return Err(error(
                    StatusCode::CONFLICT,
                    "This roll ID already belongs to a different request",
                ));
            }
            return Ok(existing.response.clone());
        }
        let mut rng = rand::thread_rng();
        let groups = request
            .groups
            .iter()
            .map(|g| evaluate(g, &mut rng))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| error(StatusCode::BAD_REQUEST, &e))?;
        let response = Response {
            id: request.id.clone(),
            total: groups.iter().map(|g| g.total).sum(),
            groups,
        };
        if self.0.len() >= 512 {
            self.0.pop_front();
        }
        self.0.push_back(Cached {
            id: request.id.clone(),
            groups: request.groups.clone(),
            response: response.clone(),
        });
        Ok(response)
    }
}
async fn roll(
    State(cache): State<Shared>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, Error> {
    if request.id.is_empty()
        || request.id.len() > 128
        || request.groups.is_empty()
        || request.groups.len() > 32
        || request.groups.iter().any(|g| g.label.len() > 100)
        || request.delay_ms > 5000
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Request exceeds prototype limits",
        ));
    }
    if request.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(request.delay_ms)).await;
    }
    if request.fail {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Could not roll. Try again.",
        ));
    }
    cache
        .lock()
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Roll state unavailable"))?
        .roll(&request)
        .map(Json)
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/roll", post(roll))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .with_state(Shared::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8821").await?;
    eprintln!("Dice prototype API: http://127.0.0.1:8821 (in-memory; no encounter writes)");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    #[test]
    fn breakdown_explains_discarded_faces_without_markup() {
        let dice = [
            Die {
                sides: 20,
                value: 17,
                kept: true,
            },
            Die {
                sides: 20,
                value: 4,
                kept: false,
            },
        ];
        assert_eq!(
            format_breakdown(&dice, 8, 25),
            "d20: 17, d20: 4 (discarded); modifier +8; total 25"
        );
    }
    fn group(formula: &str) -> Group {
        Group {
            label: "test".into(),
            formula: formula.into(),
        }
    }
    #[test]
    fn evaluator_preserves_mixed_dice_and_modifier() {
        let result = evaluate(
            &group("1d8+1d6+4"),
            &mut rand::rngs::StdRng::seed_from_u64(42),
        )
        .unwrap();
        assert_eq!(
            result.dice.iter().map(|d| d.sides).collect::<Vec<_>>(),
            vec![8, 6]
        );
        assert_eq!(
            result.total,
            result.dice.iter().map(|d| d.value as i64).sum::<i64>() + 4
        );
    }
    #[test]
    fn keep_retains_discarded_faces_and_one_winner_even_on_ties() {
        for seed in 0..500 {
            for (formula, high) in [("2d20K1+8", true), ("2d20k1+8", false)] {
                let r = evaluate(
                    &group(formula),
                    &mut rand::rngs::StdRng::seed_from_u64(seed),
                )
                .unwrap();
                assert_eq!(r.dice.len(), 2);
                assert_eq!(r.dice.iter().filter(|d| d.kept).count(), 1);
                let expected = if high {
                    r.dice.iter().map(|d| d.value).max()
                } else {
                    r.dice.iter().map(|d| d.value).min()
                }
                .unwrap();
                assert_eq!(r.total, expected as i64 + 8);
                assert_eq!(r.dice.iter().find(|d| d.kept).unwrap().value, expected);
            }
        }
    }
    #[test]
    fn rejects_unbounded_and_nonpreset_formulas() {
        for formula in [
            "100000d20",
            "1d6!",
            "1d20+101",
            "1d20-101",
            "@actor.level",
            "1d20/0",
        ] {
            assert!(preset(formula).is_err());
        }
    }
    #[test]
    fn retry_is_same_roll_and_conflicting_payload_is_rejected() {
        let mut cache = Cache::default();
        let mut request = Request {
            id: "retry".into(),
            groups: vec![group("2d6+4")],
            delay_ms: 0,
            fail: false,
        };
        let first = cache.roll(&request).unwrap();
        assert_eq!(first, cache.roll(&request).unwrap());
        request.groups = vec![group("1d20")];
        assert_eq!(cache.roll(&request).unwrap_err().0, StatusCode::CONFLICT);
    }
    #[test]
    fn dicexp_probe_confirms_numeric_result_without_face_history() {
        let mut bag = dicexp::DiceBag::new(dicexp::simple_rng(42));
        let result = bag.eval("1d8+1d6+4").unwrap();
        assert_eq!((result.min, result.max), (6, 18));
        assert!((6..=18).contains(&result.total));
        assert!(bag.eval("2d20K1").is_err());
    }
}
