pub fn create_sql(dimensions: usize) -> String {
    format!("CREATE VIRTUAL TABLE record_vector_index USING vec0(embedding FLOAT[{dimensions}])")
}

pub fn insert_sql() -> String {
    "INSERT INTO record_vector_index (rowid, embedding) VALUES (?1, ?2)".to_string()
}
