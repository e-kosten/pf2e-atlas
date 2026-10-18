use crate::DiceError;
use diceng::{BinaryOp, DiceAtom, Expression, FilterDirection, FilterType};

const MAX_VALUE: i64 = 1_000_000_000;

/// Prove safe integer bounds before entering the library's unchecked arithmetic.
/// This validates its AST; it is not an alternative formula parser or evaluator.
pub(crate) fn validate(
    expr: &Expression,
    depth: usize,
    count: &mut u32,
) -> Result<(i64, i64), DiceError> {
    if depth > 32 {
        return Err(DiceError::Limit);
    }
    let bounds = match expr {
        Expression::Literal(n) => (i64::from(*n), i64::from(*n)),
        Expression::Dice(d) => {
            let (n, sides) = match d.atom {
                DiceAtom::Standard { count, sides } => (count, sides),
                DiceAtom::Percent { count } => (count, 100),
                _ => return Err(DiceError::Unsupported),
            };
            if n == 0 || n > 200 || !(2..=1000).contains(&sides) {
                return Err(DiceError::Limit);
            }
            *count += n;
            if *count > 200 {
                return Err(DiceError::Limit);
            }
            if !d.functors.is_empty()
                || d.count_threshold.is_some()
                || d.sort_order.is_some()
                || d.filters.len() > 1
            {
                return Err(DiceError::Unsupported);
            }
            let kept = if let Some(filter) = d.filters.first() {
                if filter.filter_type != FilterType::Keep
                    || !matches!(
                        filter.direction,
                        FilterDirection::Highest | FilterDirection::Lowest
                    )
                {
                    return Err(DiceError::Unsupported);
                }
                if filter.n == 0 || filter.n > n {
                    return Err(DiceError::Limit);
                }
                filter.n
            } else {
                n
            };
            (i64::from(kept), i64::from(kept) * i64::from(sides))
        }
        Expression::UnaryMinus(inner) => {
            let (lo, hi) = validate(inner, depth + 1, count)?;
            (-hi, -lo)
        }
        Expression::BinaryOp { op, left, right } => {
            let (a, b) = validate(left, depth + 1, count)?;
            let (c, d) = validate(right, depth + 1, count)?;
            match op {
                BinaryOp::Add => (a + c, b + d),
                BinaryOp::Sub => (a - d, b - c),
                BinaryOp::Mul => {
                    let values = [a * c, a * d, b * c, b * d];
                    extrema(values)
                }
                BinaryOp::Div => {
                    if c <= 0 && d >= 0 {
                        return Err(DiceError::Invalid("Divisor can be zero".into()));
                    }
                    extrema([a / c, a / d, b / c, b / d])
                }
            }
        }
        _ => return Err(DiceError::Unsupported),
    };
    if bounds.0 < -MAX_VALUE || bounds.1 > MAX_VALUE {
        return Err(DiceError::Limit);
    }
    Ok(bounds)
}

fn extrema(values: [i64; 4]) -> (i64, i64) {
    values
        .into_iter()
        .fold((i64::MAX, i64::MIN), |(lo, hi), n| (lo.min(n), hi.max(n)))
}
