//! Sense availability is evaluated for a stated target; range is never assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Precision {
    Precise,
    Imprecise,
    Vague,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SenseFacts {
    pub visual: bool,
    pub auditory: bool,
    pub precision: Option<Precision>,
    pub range_feet: Option<f64>,
    pub available: Option<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum Availability {
    Available,
    Unavailable,
    NeedsContext,
}

pub(in crate::encounters) fn availability(
    sense: SenseFacts,
    blinded: bool,
    distance_feet: Option<f64>,
) -> Availability {
    if sense.available == Some(false) || (blinded && sense.visual) {
        return Availability::Unavailable;
    }
    if let (Some(range), Some(distance)) = (sense.range_feet, distance_feet) {
        if distance > range {
            return Availability::Unavailable;
        }
    } else if sense.range_feet.is_some() {
        return Availability::NeedsContext;
    }
    match sense.available {
        Some(true) => Availability::Available,
        _ => Availability::NeedsContext,
    }
}

/// Blinded's nonvisual penalty is unresolved when a precise alternative may apply.
pub(in crate::encounters) fn blinded_penalty_applies(
    senses: &[SenseFacts],
    distance_feet: Option<f64>,
) -> Option<bool> {
    let mut uncertain = false;
    for sense in senses {
        if sense.visual || sense.precision != Some(Precision::Precise) {
            continue;
        }
        match availability(*sense, true, distance_feet) {
            Availability::Available => return Some(false),
            Availability::NeedsContext => uncertain = true,
            Availability::Unavailable => {}
        }
    }
    (!uncertain).then_some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precise_echolocation_requires_range_context() {
        let sense = SenseFacts {
            visual: false,
            auditory: true,
            precision: Some(Precision::Precise),
            range_feet: Some(40.0),
            available: Some(true),
        };
        for (distance, expected) in [
            (None, None),
            (Some(39.0), Some(false)),
            (Some(40.0), Some(false)),
            (Some(41.0), Some(true)),
        ] {
            assert_eq!(blinded_penalty_applies(&[sense], distance), expected);
        }
    }
    #[test]
    fn blinded_disables_vision_without_disabling_hearing() {
        let hearing = SenseFacts {
            visual: false,
            auditory: true,
            precision: Some(Precision::Imprecise),
            range_feet: None,
            available: Some(true),
        };
        assert_eq!(availability(hearing, true, None), Availability::Available);
        assert_eq!(
            availability(
                SenseFacts {
                    visual: true,
                    ..hearing
                },
                true,
                None
            ),
            Availability::Unavailable
        );
        assert_eq!(blinded_penalty_applies(&[hearing], None), Some(true));
    }
}
