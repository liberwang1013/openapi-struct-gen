use std::collections::BTreeMap;

use openapiv3::{
    MediaType, OpenAPI, PathItem, ReferenceOr, RequestBody, Response, Schema, SchemaKind,
    StatusCode, Type,
};

use codegen::Scope;

//scope.new_struct("Foo")
//    .derive("Debug")
//    .field("one", "usize")
//    .field("two", "String");

pub fn generate(schemas: BTreeMap<String, Schema>) -> String {
    let mut scope = Scope::new();
    for (name, schema) in schemas.into_iter() {
        generate_for_schema(&mut scope, name, schema);
    }
    scope.to_string()
}

fn generate_for_schema(scope: &mut Scope, name: String, schema: Schema) {
    match schema.schema_kind {
        SchemaKind::Type(r#type) => generate_struct(scope, name, r#type),
        SchemaKind::OneOf { one_of } => generate_enum(scope, name, one_of),
        SchemaKind::AnyOf { any_of } => generate_enum(scope, name, any_of),
        _ => panic!("Does not support 'allOf', 'not' and 'any'"),
    }
}

fn generate_struct(scope: &mut Scope, name: String, r#type: Type) {
    println!("#{:#?}", r#type);
    todo!();
}

fn generate_enum(scope: &mut Scope, name: String, types: Vec<ReferenceOr<Schema>>) {
    println!("#{:#?}", types);
    todo!();
}
