fn main() -> Result<(), serde_json::Error> {
    println!("{}", atlas_web::openapi().to_pretty_json()?);
    Ok(())
}
