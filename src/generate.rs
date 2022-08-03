use std::collections::{BTreeMap, HashSet};

use codegen::Scope;
use openapiv3::{
    ArrayType, IntegerFormat, IntegerType, MediaType, NumberFormat, NumberType, OpenAPI, PathItem,
    ReferenceOr, RequestBody, Response, Schema, SchemaKind, StatusCode, Type,
    VariantOrUnknownOrEmpty,
};

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

fn get_number_type(t: NumberType) -> String {
    if let VariantOrUnknownOrEmpty::Item(f) = t.format {
        if f == NumberFormat::Double {
            "f64".into()
        } else {
            "f32".into()
        }
    } else {
        "f32".into()
    }
}

fn get_integer_type(t: IntegerType) -> String {
    if let VariantOrUnknownOrEmpty::Item(f) = t.format {
        if f == IntegerFormat::Int64 {
            "i64".into()
        } else {
            "i32".into()
        }
    } else {
        "i32".into()
    }
}

fn gen_property_type_for_schema_kind(sk: SchemaKind) -> String {
    let t = match sk {
        SchemaKind::Type(r#type) => r#type,
        _ => panic!("Does not support 'oneOf', 'anyOf' 'allOf', 'not' and 'any'"),
    };
    match t {
        Type::String(_) => "String".into(),
        Type::Number(f) => get_number_type(f),
        Type::Integer(f) => get_integer_type(f),
        Type::Object(_) => todo!(),
        Type::Array(a) => gen_array_type(a),
        Type::Boolean {} => "bool".into(),
    }
}

fn get_property_type_from_schema_refor(
    refor: ReferenceOr<Box<Schema>>,
    is_required: bool,
) -> String {
    let t = match refor {
        ReferenceOr::Item(i) => gen_property_type_for_schema_kind(i.schema_kind),
        ReferenceOr::Reference { reference } => handle_reference(reference),
    };
    if is_required {
        t
    } else {
        format!("Option<{}>", t)
    }
}

fn gen_array_type(a: ArrayType) -> String {
    let inner_type = if let Some(items) = a.items {
        get_property_type_from_schema_refor(items, true)
    } else {
        todo!();
    };
    format!("Vec<{}>", inner_type)
}

fn handle_reference(reference: String) -> String {
    let mut split = reference.split("/").into_iter().collect::<Vec<_>>();
    if split[0] != "#" {
        unreachable!();
    }
    if split[1] != "components" {
        panic!("Trying to load from something other than components");
    }
    if split[2] != "schemas" {
        panic!("Only references to schemas are supported");
    }
    split.pop().unwrap().to_owned()
}

fn generate_struct(scope: &mut Scope, name: String, r#type: Type) {
    match r#type {
        Type::Object(obj) => {
            let mut r#struct = scope.new_struct(&name).vis("pub").derive("Debug");
            let required = obj.required.into_iter().collect::<HashSet<String>>();
            for (name, refor) in obj.properties {
                let is_required = required.contains(&name);
                let t = get_property_type_from_schema_refor(refor, is_required);
                r#struct.field(&name, &t);
            }
        }
        Type::Array(a) => {
            scope.raw(&format!("pub type {} = {};", name, gen_array_type(a)));
        }
        t => {
            println!("#{:#?}", t);
            unreachable!();
        }
    }
}

fn generate_enum(scope: &mut Scope, name: String, types: Vec<ReferenceOr<Schema>>) {
    println!("#{:#?}", types);
    todo!();
}
