//! Shape identity for source unions, separate from ordinary pre-default object fields.
use super::parse::{ParseResult, SourceContext};
use super::value::SourceValue;

pub(super) fn union_object(value: &SourceValue, required: &[&str]) -> bool {
    matches!(value, SourceValue::Object(_))
        && required
            .iter()
            .all(|name| union_member(value, name, |_| true))
}

pub(super) fn union_member(
    value: &SourceValue,
    name: &str,
    matches: impl Fn(&SourceValue) -> bool,
) -> bool {
    matches!(value, SourceValue::Object(object) if object.fields.iter().any(|(key,value)| key == name && matches(value)))
}

pub(super) fn union_required(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
    required: &[(&str, bool)],
) -> ParseResult<()> {
    let SourceValue::Object(object) = value else {
        return Err(context.error(path, "union object", value));
    };
    for (name, nullable) in required {
        let member_path = format!("{path}.{name}");
        let mut values = object.fields.iter().filter(|(key, _)| key == name);
        let Some((_, value)) = values.next() else {
            return Err(context.message(&member_path, "present union identity member", "missing"));
        };
        if values.next().is_some() {
            return Err(context.message(
                &member_path,
                "one structural member",
                "duplicate members",
            ));
        }
        if !nullable && matches!(value, SourceValue::Null) {
            return Err(context.error(&member_path, "non-null union identity member", value));
        }
    }
    Ok(())
}

pub(super) struct UnionCandidates<T> {
    names: Vec<&'static str>,
    first: Option<ParseResult<T>>,
}
impl<T> UnionCandidates<T> {
    pub(super) fn new() -> Self {
        Self {
            names: Vec::new(),
            first: None,
        }
    }
    pub(super) fn push(&mut self, name: &'static str, parse: impl FnOnce() -> ParseResult<T>) {
        self.names.push(name);
        if self.names.len() == 1 {
            self.first = Some(parse());
        }
    }
    pub(super) fn finish(
        self,
        value: &SourceValue,
        context: &SourceContext,
        path: &str,
    ) -> ParseResult<T> {
        match (self.names.len(), self.first) {
            (1, Some(result)) => result,
            (0, _) => Err(context.error(path, "exactly one union alternative", value)),
            _ => Err(context.message(
                path,
                "exactly one union alternative",
                self.names.join(" | "),
            )),
        }
    }
}
