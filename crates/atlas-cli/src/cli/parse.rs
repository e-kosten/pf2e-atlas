use atlas_domain::DetailLevel;

pub(crate) const DETAIL_HELP: &str = "Record detail level: summary identity; preview compact scan and teaser; description authored content without mechanics; standard complete mechanics and teaser; full mechanics, rich content, and concise provenance";

pub(crate) fn parse_detail_level(value: &str) -> Result<DetailLevel, String> {
    value
        .parse::<DetailLevel>()
        .map_err(|error| error.to_string())
}
