use atlas_domain::DetailLevel;

pub(crate) const DETAIL_HELP: &str = "Record detail level: summary, preview, description, standard, or full; preview includes compact scan facts and a truncated description";

pub(crate) fn parse_detail_level(value: &str) -> Result<DetailLevel, String> {
    value
        .parse::<DetailLevel>()
        .map_err(|error| error.to_string())
}
