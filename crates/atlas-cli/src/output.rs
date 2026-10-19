use atlas_cli_support::write_json_data;
use atlas_index::ArtifactValidationReport;
use std::process::ExitCode;
pub(crate) fn write_validation_report(
    report: ArtifactValidationReport,
    json: bool,
) -> Result<ExitCode, String> {
    if json {
        write_json_data(&report)?;
    } else {
        println!(
            "ok: checked {} source snapshots, {} prepared fields, {} projections, {} lexical units, {} semantic units",
            report.checked_snapshots,
            report.prepared_fields,
            report.projection_rows,
            report.lexical_units,
            report.semantic_units
        );
    }
    Ok(ExitCode::SUCCESS)
}
