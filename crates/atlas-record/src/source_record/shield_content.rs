//! Recognize the authored two-row durability table without changing its content.
use super::ItemSourceView;
use scraper::{Html, Selector};

impl ItemSourceView<'_> {
    /// Suppress duplicate numeric summaries only when the preserved table's
    /// hardness and maximum HP exactly match the typed source facts. BT stays
    /// authored; this policy neither computes it nor removes the source table.
    pub fn shield_durability_in_table(self) -> bool {
        let Self::ShieldSource(source) = self else {
            return false;
        };
        let Some(system) = source.system.as_value() else {
            return false;
        };
        let Some(html) = system
            .description
            .as_value()
            .and_then(|d| d.value.as_value())
        else {
            return false;
        };
        let Some(hardness) = system.hardness.as_value().and_then(|v| v.as_f64()) else {
            return false;
        };
        let Some(hp) = system
            .hp
            .as_value()
            .and_then(|v| v.max.as_value())
            .and_then(|v| v.as_f64())
        else {
            return false;
        };
        let (Ok(tables), Ok(rows), Ok(cells)) = (
            Selector::parse("table"),
            Selector::parse("tr"),
            Selector::parse("th, td"),
        ) else {
            return false;
        };
        Html::parse_fragment(html).select(&tables).any(|table| {
            let rows: Vec<_> = table.select(&rows).collect();
            if rows.len() != 2 {
                return false;
            }
            let labels: Vec<_> = rows[0]
                .select(&cells)
                .map(|c| c.text().collect::<String>().trim().to_owned())
                .collect();
            let values: Vec<_> = rows[1]
                .select(&cells)
                .map(|c| c.text().collect::<String>().trim().to_owned())
                .collect();
            labels == ["Hardness", "HP", "BT"]
                && values.len() == 3
                && values[0].parse::<f64>().ok() == Some(hardness)
                && values[1].parse::<f64>().ok() == Some(hp)
        })
    }
}
