#[cfg(feature = "build")]
pub mod error;
#[cfg(feature = "build")]
mod generate;
#[cfg(feature = "build")]
mod parse;

#[cfg(feature = "build")]
use crate::error::GenError;
#[cfg(feature = "build")]
use openapiv3::OpenAPI;

#[cfg(feature = "build")]
pub fn generate<P1: AsRef<std::path::Path>, P2: AsRef<std::path::Path>>(
    schema_filename: P1,
    output_filename: P2,
    derivatives: Option<&[&str]>,
    imports: Option<&[(&str, &str)]>,
    container_attrs: Option<&[(&str, Option<&[&str]>)]>,
    field_attrs: Option<&[(&str, Option<&[&str]>)]>,
) -> Result<(), GenError> {
    let schema_filename = schema_filename.as_ref();
    let data = std::fs::read_to_string(schema_filename)?;
    let oapi: OpenAPI = match schema_filename.extension().map(|s| s.to_str().unwrap()) {
        Some("json") => serde_json::from_str(&data)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&data)?,
        o => return Err(GenError::WrongFileExtension(o.map(|s| s.to_owned()))),
    };
    let schemas_map = parse::parse_schema(oapi);
    let resp = generate::generate(
        schemas_map,
        derivatives,
        imports,
        container_attrs,
        field_attrs,
    );
    std::fs::write(output_filename, resp)?;
    Ok(())
}

#[macro_export]
macro_rules! include {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}
