use super::FormulaContext;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Dice,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Function {
    Ceil,
    Floor,
    Min,
    Max,
    Resolve,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Expr {
    Number(f64),
    Variable(String),
    Negate(Box<Self>),
    Binary(Op, Box<Self>, Box<Self>),
    Call(Function, Vec<Self>),
}

impl Expr {
    pub fn parse(input: &str) -> Option<Self> {
        if input.len() > 4096 {
            return None;
        }
        let mut parser = Parser {
            input: input.as_bytes(),
            index: 0,
            nodes: 0,
        };
        let value = parser.expression(0, 0)?;
        parser.space();
        (parser.index == parser.input.len()).then_some(value)
    }
    pub fn substitute(&self, context: &FormulaContext) -> Option<Self> {
        let result = match self {
            Self::Number(n) => Self::Number(*n),
            Self::Variable(name) => context
                .value(name)
                .map(Self::Number)
                .unwrap_or_else(|| self.clone()),
            Self::Negate(value) => {
                let value = value.substitute(context)?;
                match value {
                    Self::Number(n) => Self::Number(-n),
                    _ => Self::Negate(Box::new(value)),
                }
            }
            Self::Binary(op, left, right) => {
                let left = left.substitute(context)?;
                let right = right.substitute(context)?;
                match (&left, &right) {
                    (Self::Number(a), Self::Number(b)) if *op != Op::Dice => {
                        Self::Number(match op {
                            Op::Add => a + b,
                            Op::Subtract => a - b,
                            Op::Multiply => a * b,
                            Op::Divide if *b != 0.0 => a / b,
                            _ => return None,
                        })
                    }
                    _ => Self::Binary(*op, Box::new(left), Box::new(right)),
                }
            }
            Self::Call(function, arguments) => {
                let arguments = arguments
                    .iter()
                    .map(|a| a.substitute(context))
                    .collect::<Option<Vec<_>>>()?;
                let numbers = arguments
                    .iter()
                    .map(|a| match a {
                        Self::Number(n) => Some(*n),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>();
                if let Some(numbers) = numbers {
                    Self::Number(match function {
                        Function::Ceil => numbers[0].ceil(),
                        Function::Floor => numbers[0].floor(),
                        Function::Resolve => numbers[0],
                        Function::Min => numbers.into_iter().reduce(f64::min)?,
                        Function::Max => numbers.into_iter().reduce(f64::max)?,
                    })
                } else if *function == Function::Resolve {
                    arguments.into_iter().next()?
                } else {
                    Self::Call(*function, arguments)
                }
            }
        };
        if matches!(&result, Self::Number(n) if !n.is_finite() || n.abs() > 1_000_000_000.0) {
            None
        } else {
            Some(result)
        }
    }
    pub fn literal(&self) -> Option<String> {
        match self {
            Self::Number(n) => Some(number(*n)),
            Self::Negate(v) => Some(format!("(-{})", v.literal()?)),
            Self::Binary(Op::Dice, a, b) => Some(format!("{}d{}", a.literal()?, b.literal()?)),
            Self::Binary(op, a, b) => Some(format!(
                "({}{}{})",
                a.literal()?,
                match op {
                    Op::Add => "+",
                    Op::Subtract => "-",
                    Op::Multiply => "*",
                    Op::Divide => "/",
                    Op::Dice => "d",
                },
                b.literal()?
            )),
            _ => None,
        }
    }
    pub fn description(&self, context: &FormulaContext) -> String {
        match self {
            Self::Number(n) => number(*n),
            Self::Variable(name) => context.label(name),
            Self::Negate(v) => format!("−({})", v.description(context)),
            Self::Binary(Op::Divide, a, b) if **b == Self::Number(2.0) => {
                format!("half {}", a.grouped(context))
            }
            Self::Binary(Op::Dice, a, b) if self.literal().is_some() => {
                format!("{}d{}", a.description(context), b.description(context))
            }
            Self::Binary(Op::Dice, a, b) => {
                format!("({})d{}", a.description(context), b.description(context))
            }
            Self::Binary(op, a, b) => format!(
                "{} {} {}",
                a.grouped(context),
                match op {
                    Op::Add => "+",
                    Op::Subtract => "−",
                    Op::Multiply => "×",
                    Op::Divide => "÷",
                    Op::Dice => "dice with",
                },
                b.grouped(context)
            ),
            Self::Call(Function::Ceil, a) => format!("{} (rounded up)", a[0].grouped(context)),
            Self::Call(Function::Floor, a) => {
                format!("{} (rounded down)", a[0].grouped(context))
            }
            Self::Call(Function::Resolve, a) => a[0].description(context),
            Self::Call(f, arguments) => {
                if arguments.len() == 2
                    && let Self::Number(n) = arguments[0]
                {
                    return format!(
                        "{} ({} {})",
                        arguments[1].description(context),
                        if *f == Function::Min {
                            "maximum"
                        } else {
                            "minimum"
                        },
                        number(n)
                    );
                }
                format!(
                    "the {} of {}",
                    if *f == Function::Min {
                        "minimum"
                    } else {
                        "maximum"
                    },
                    arguments
                        .iter()
                        .map(|a| a.description(context))
                        .collect::<Vec<_>>()
                        .join(" and ")
                )
            }
        }
    }
    fn grouped(&self, context: &FormulaContext) -> String {
        let text = self.description(context);
        if matches!(self, Self::Binary(op,..) if *op != Op::Dice) {
            format!("({text})")
        } else {
            text
        }
    }
}
pub(super) fn number(value: f64) -> String {
    if value == 0.0 {
        "0".into()
    } else {
        value.to_string()
    }
}
struct Parser<'a> {
    input: &'a [u8],
    index: usize,
    nodes: usize,
}
impl Parser<'_> {
    fn space(&mut self) {
        while self
            .input
            .get(self.index)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.index += 1;
        }
    }
    fn take(&mut self, value: u8) -> bool {
        self.space();
        if self.input.get(self.index) == Some(&value) {
            self.index += 1;
            true
        } else {
            false
        }
    }
    fn expression(&mut self, minimum: u8, depth: usize) -> Option<Expr> {
        if depth > 32 || self.nodes >= 256 {
            return None;
        }
        self.nodes += 1;
        self.space();
        let mut left = if self.take(b'(') {
            let v = self.expression(0, depth + 1)?;
            if !self.take(b')') {
                return None;
            }
            v
        } else if self.take(b'-') {
            Expr::Negate(Box::new(self.expression(40, depth + 1)?))
        } else if self.take(b'+') {
            self.expression(40, depth + 1)?
        } else if self.take(b'@') {
            let start = self.index;
            while self
                .input
                .get(self.index)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'.' || *c == b'_')
            {
                self.index += 1;
            }
            let name = std::str::from_utf8(&self.input[start..self.index]).ok()?;
            if !matches!(
                name,
                "actor.level"
                    | "item.rank"
                    | "item.level"
                    | "item.badge.value"
                    | "kingdom.control.dc"
                    | "self.level"
            ) {
                return None;
            }
            Expr::Variable(format!("@{name}"))
        } else if self
            .input
            .get(self.index)
            .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
        {
            let start = self.index;
            while self
                .input
                .get(self.index)
                .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
            {
                self.index += 1;
            }
            let n = std::str::from_utf8(&self.input[start..self.index])
                .ok()?
                .parse::<f64>()
                .ok()?;
            if !n.is_finite() || n.abs() > 1_000_000_000.0 {
                return None;
            }
            Expr::Number(n)
        } else {
            let start = self.index;
            while self
                .input
                .get(self.index)
                .is_some_and(u8::is_ascii_alphabetic)
            {
                self.index += 1;
            }
            let function = match &self.input[start..self.index] {
                b"ceil" => Function::Ceil,
                b"floor" => Function::Floor,
                b"min" => Function::Min,
                b"max" => Function::Max,
                b"resolve" => Function::Resolve,
                _ => return None,
            };
            if !self.take(b'(') {
                return None;
            }
            let mut args = vec![self.expression(0, depth + 1)?];
            while self.take(b',') {
                if args.len() >= 8 {
                    return None;
                }
                args.push(self.expression(0, depth + 1)?);
            }
            if !self.take(b')')
                || (!matches!(function, Function::Min | Function::Max) && args.len() != 1)
            {
                return None;
            }
            Expr::Call(function, args)
        };
        loop {
            self.space();
            let (op, precedence) = match self.input.get(self.index) {
                Some(b'+') => (Op::Add, 10),
                Some(b'-') => (Op::Subtract, 10),
                Some(b'*') => (Op::Multiply, 20),
                Some(b'/') => (Op::Divide, 20),
                Some(b'd' | b'D') => (Op::Dice, 30),
                _ => break,
            };
            if precedence < minimum {
                break;
            }
            self.index += 1;
            left = Expr::Binary(
                op,
                Box::new(left),
                Box::new(self.expression(precedence + 1, depth + 1)?),
            );
        }
        Some(left)
    }
}
