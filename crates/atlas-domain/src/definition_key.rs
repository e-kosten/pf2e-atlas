use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An inventoried property namespace, independent of Foundry document identity.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionNamespace {
    Trait,
    Rarity,
    Sense,
    Language,
    Domain,
    WeaponGroup,
    ArmorGroup,
    DamageType,
    ImmunityType,
    WeaknessType,
    ResistanceType,
    Skill,
    Ability,
    Size,
    Tradition,
    Rule,
}

impl DefinitionNamespace {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trait => "trait",
            Self::Rarity => "rarity",
            Self::Sense => "sense",
            Self::Language => "language",
            Self::Domain => "domain",
            Self::WeaponGroup => "weapon_group",
            Self::ArmorGroup => "armor_group",
            Self::DamageType => "damage_type",
            Self::ImmunityType => "immunity_type",
            Self::WeaknessType => "weakness_type",
            Self::ResistanceType => "resistance_type",
            Self::Skill => "skill",
            Self::Ability => "ability",
            Self::Size => "size",
            Self::Tradition => "tradition",
            Self::Rule => "rule",
        }
    }
}

/// Exact vocabulary/source-rule identity. Labels never determine this key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[ts(type = "string")]
pub struct DefinitionKey {
    namespace: DefinitionNamespace,
    identifier: String,
}
#[cfg(feature = "openapi")]
impl utoipa::ToSchema for DefinitionKey {}
#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for DefinitionKey {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        <String as utoipa::PartialSchema>::schema()
    }
}

impl DefinitionKey {
    pub fn new(
        namespace: DefinitionNamespace,
        identifier: impl Into<String>,
    ) -> Result<Self, DefinitionKeyParseError> {
        let identifier = identifier.into();
        if identifier.is_empty()
            || identifier
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
            || (namespace != DefinitionNamespace::Rule && identifier.contains(':'))
        {
            return Err(DefinitionKeyParseError::InvalidIdentifier);
        }
        if namespace == DefinitionNamespace::Rule {
            let (package, key) = identifier
                .split_once(':')
                .ok_or(DefinitionKeyParseError::InvalidSourceRule)?;
            if package.is_empty()
                || key.is_empty()
                || !package
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            {
                return Err(DefinitionKeyParseError::InvalidSourceRule);
            }
        }
        Ok(Self {
            namespace,
            identifier,
        })
    }

    pub fn namespace(&self) -> DefinitionNamespace {
        self.namespace
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn trait_key(&self) -> Option<crate::TraitKey> {
        if self.namespace == DefinitionNamespace::Trait {
            crate::TraitKey::new(self.identifier.clone()).ok()
        } else {
            None
        }
    }
}

impl From<crate::TraitKey> for DefinitionKey {
    fn from(key: crate::TraitKey) -> Self {
        Self {
            namespace: DefinitionNamespace::Trait,
            identifier: key.identifier().to_owned(),
        }
    }
}

impl fmt::Display for DefinitionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace.as_str(), self.identifier)
    }
}

impl FromStr for DefinitionKey {
    type Err = DefinitionKeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (namespace, identifier) = value.split_once(':').ok_or(Self::Err::InvalidNamespace)?;
        let namespace = match namespace {
            "trait" => DefinitionNamespace::Trait,
            "rarity" => DefinitionNamespace::Rarity,
            "sense" => DefinitionNamespace::Sense,
            "language" => DefinitionNamespace::Language,
            "domain" => DefinitionNamespace::Domain,
            "weapon_group" => DefinitionNamespace::WeaponGroup,
            "armor_group" => DefinitionNamespace::ArmorGroup,
            "damage_type" => DefinitionNamespace::DamageType,
            "immunity_type" => DefinitionNamespace::ImmunityType,
            "weakness_type" => DefinitionNamespace::WeaknessType,
            "resistance_type" => DefinitionNamespace::ResistanceType,
            "skill" => DefinitionNamespace::Skill,
            "ability" => DefinitionNamespace::Ability,
            "size" => DefinitionNamespace::Size,
            "tradition" => DefinitionNamespace::Tradition,
            "rule" => DefinitionNamespace::Rule,
            _ => return Err(Self::Err::InvalidNamespace),
        };
        Self::new(namespace, identifier)
    }
}

impl Serialize for DefinitionKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DefinitionKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DefinitionKeyParseError {
    #[error("definition key must use an inventoried namespace")]
    InvalidNamespace,
    #[error(
        "definition identifier must be nonempty and contain no whitespace, control characters or extra colon"
    )]
    InvalidIdentifier,
    #[error("source-rule identity must use rule:<package>:<exact-localization-key> syntax")]
    InvalidSourceRule,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_typed_tokens_and_complete_source_rule_tail() {
        for value in [
            "trait:deadly-d8",
            "language:custom.Language",
            "domain:airapocryphal",
            "rule:pf2e:PF2E.NPC.Abilities.Glossary.SneakAttack:variant",
        ] {
            let key: DefinitionKey = value.parse().unwrap();
            assert_eq!(key.to_string(), value);
            assert_eq!(
                serde_json::from_str::<DefinitionKey>(&serde_json::to_string(&key).unwrap())
                    .unwrap(),
                key
            );
        }
        let trait_key = crate::TraitKey::new("agile").unwrap();
        assert_eq!(
            DefinitionKey::from(trait_key.clone()).trait_key(),
            Some(trait_key)
        );
        for value in [
            "language:",
            "sense:has space",
            "language:a:b",
            "record:abc",
            "rule::key",
            "rule:pf2e:",
            "rule:bad/package:key",
        ] {
            assert!(value.parse::<DefinitionKey>().is_err(), "{value}");
        }
    }
}
