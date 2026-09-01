use serde::{Deserialize, Deserializer, Serializer};

pub(crate) const JS_SAFE_INTEGER_MAX: i64 = 9_007_199_254_740_991;
pub(crate) const JS_SAFE_INTEGER_MIN: i64 = -JS_SAFE_INTEGER_MAX;

fn is_safe(value: i64) -> bool {
    (JS_SAFE_INTEGER_MIN..=JS_SAFE_INTEGER_MAX).contains(&value)
}

fn invalid_integer(value: i64) -> String {
    format!("integer {value} exceeds the JavaScript safe-integer range")
}

pub(crate) fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if !is_safe(*value) {
        return Err(serde::ser::Error::custom(invalid_integer(*value)));
    }
    serializer.serialize_i64(*value)
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    if !is_safe(value) {
        return Err(serde::de::Error::custom(invalid_integer(value)));
    }
    Ok(value)
}

pub(crate) mod optional {
    use super::{invalid_integer, is_safe};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(crate) fn serialize<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(value) = value
            && !is_safe(*value)
        {
            return Err(serde::ser::Error::custom(invalid_integer(*value)));
        }
        value.serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<i64>::deserialize(deserializer)?;
        if let Some(value) = value
            && !is_safe(value)
        {
            return Err(serde::de::Error::custom(invalid_integer(value)));
        }
        Ok(value)
    }
}
