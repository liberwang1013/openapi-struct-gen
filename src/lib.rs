pub mod error;
mod generate;
mod parse;

use crate::error::GenError;
use openapiv3::OpenAPI;

pub fn generate<P1: AsRef<std::path::Path>, P2: AsRef<std::path::Path>>(
    schema_filename: P1,
    output_filename: P2,
) -> Result<(), GenError> {
    let schema_filename = schema_filename.as_ref();
    let data = std::fs::read_to_string(schema_filename)?;
    let oapi: OpenAPI = match schema_filename.extension().map(|s| s.to_str().unwrap()) {
        Some("json") => serde_json::from_str(&data)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&data)?,
        o => return Err(GenError::WrongFileExtension(o.map(|s| s.to_owned()))),
    };
    let schemas_map = parse::parse_schema(oapi);
    let resp = generate::generate(schemas_map);
    std::fs::write(output_filename, resp)?;
    Ok(())
}
