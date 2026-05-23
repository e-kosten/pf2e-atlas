use std::fs;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use arrow_array::{
    ArrayRef, BooleanArray, FixedSizeListArray, Float32Array, Float64Array, Int64Array,
    RecordBatch, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use lbug::Connection;
use parquet::arrow::ArrowWriter;

use crate::IndexWriteError;
use crate::ladybug::writer::output::{ladybug_progress_message, ladybug_write_error};
use crate::writer_progress::elapsed_display;

pub(crate) fn copy_from_parquet(
    connection: &Connection<'_>,
    table: &str,
    path: &Path,
) -> Result<(), IndexWriteError> {
    let started_at = Instant::now();
    ladybug_progress_message(
        "ladybug_write",
        format_args!("Copying LadybugDB table {table} from Parquet"),
    );
    connection
        .query(&format!(
            "COPY {table} FROM {};",
            cypher_string_literal(&path.to_string_lossy())
        ))
        .map_err(ladybug_write_error)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Copied LadybugDB table {table} from Parquet in {}",
            elapsed_display(started_at)
        ),
    );
    Ok(())
}

pub(crate) fn write_parquet(
    path: &Path,
    fields: Vec<Field>,
    arrays: Vec<ArrayRef>,
) -> Result<(), IndexWriteError> {
    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema.clone(), arrays)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let file =
        File::create(path).map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut writer = ArrowWriter::try_new(file, schema, None)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    writer
        .write(&batch)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    writer
        .close()
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    Ok(())
}

pub(crate) fn arrow_field(name: &str, data_type: DataType, nullable: bool) -> Field {
    Field::new(name, data_type, nullable)
}

pub(crate) fn arrow_fixed_f32_list_field(name: &str, dimensions: usize) -> Field {
    Field::new(
        name,
        DataType::FixedSizeList(
            Arc::new(arrow_field("item", DataType::Float32, false)),
            dimensions as i32,
        ),
        false,
    )
}

pub(crate) fn arrow_strings(values: impl IntoIterator<Item = String>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values))
}

pub(crate) fn arrow_optional_strings(values: impl IntoIterator<Item = Option<String>>) -> ArrayRef {
    Arc::new(StringArray::from_iter(values))
}

pub(crate) fn arrow_ints(values: impl IntoIterator<Item = i64>) -> ArrayRef {
    Arc::new(Int64Array::from_iter_values(values))
}

pub(crate) fn arrow_optional_ints(values: impl IntoIterator<Item = Option<i64>>) -> ArrayRef {
    Arc::new(Int64Array::from_iter(values))
}

pub(crate) fn arrow_bools(values: impl IntoIterator<Item = bool>) -> ArrayRef {
    Arc::new(BooleanArray::from_iter(values.into_iter().map(Some)))
}

pub(crate) fn arrow_optional_bools(values: impl IntoIterator<Item = Option<bool>>) -> ArrayRef {
    Arc::new(BooleanArray::from_iter(values))
}

pub(crate) fn arrow_optional_floats(values: impl IntoIterator<Item = Option<f64>>) -> ArrayRef {
    Arc::new(Float64Array::from_iter(values))
}

pub(crate) fn arrow_fixed_f32_lists<'a>(
    rows: impl IntoIterator<Item = &'a [f32]>,
    dimensions: usize,
) -> Result<ArrayRef, IndexWriteError> {
    let mut values = Vec::new();
    for row in rows {
        if row.len() != dimensions {
            return Err(IndexWriteError::WriteFailed(format!(
                "LadybugDB embedding row has {} dimensions; expected {dimensions}",
                row.len()
            )));
        }
        values.extend_from_slice(row);
    }
    let values: ArrayRef = Arc::new(Float32Array::from(values));
    let field = Arc::new(arrow_field("item", DataType::Float32, false));
    Ok(Arc::new(
        FixedSizeListArray::try_new(field, dimensions as i32, values, None)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
    ) as ArrayRef)
}

pub(crate) fn recreate_dir(path: &Path) -> Result<(), IndexWriteError> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    fs::create_dir_all(path).map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn cypher_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
