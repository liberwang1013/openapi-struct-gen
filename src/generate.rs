use std::collections::{BTreeMap, HashSet};

use check_keyword::CheckKeyword;

use codegen::{Field, Scope};
use heck::ToSnekCase;
use openapiv3::{
    ArrayType, IntegerFormat, IntegerType, NumberFormat, NumberType, ReferenceOr, Schema,
    SchemaKind, Type, VariantOrUnknownOrEmpty,
};

pub fn generate(
    schemas: BTreeMap<String, Schema>,
    derivatives: Option<&[&str]>,
    imports: Option<&[(&str, &str)]>,
    container_attrs: Option<&[(&str, Option<&[&str]>)]>,
    field_attrs: Option<&[(&str, Option<&[&str]>)]>,
) -> String {
    let mut scope = Scope::new();
    if let Some(imports) = imports {
        for (path, name) in imports {
            scope.import(path, name);
        }
    }
    for (name, schema) in schemas.into_iter() {
        generate_for_schema(
            &mut scope,
            name,
            schema,
            derivatives,
            container_attrs,
            field_attrs,
        );
    }
    scope.to_string()
}

fn generate_for_schema(
    scope: &mut Scope,
    name: String,
    schema: Schema,
    derivatives: Option<&[&str]>,
    container_attrs: Option<&[(&str, Option<&[&str]>)]>,
    field_attrs: Option<&[(&str, Option<&[&str]>)]>,
) {
    match schema.schema_kind {
        SchemaKind::Type(r#type) => generate_struct(
            scope,
            name,
            r#type,
            derivatives,
            container_attrs,
            field_attrs,
        ),
        SchemaKind::OneOf { one_of } => generate_enum(
            scope,
            name,
            one_of,
            derivatives,
            container_attrs,
            field_attrs,
        ),
        SchemaKind::AnyOf { any_of } => generate_enum(
            scope,
            name,
            any_of,
            derivatives,
            container_attrs,
            field_attrs,
        ),
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

fn gen_type_name_for_type(t: Type) -> String {
    match t {
        Type::String(_) => "String".into(),
        Type::Number(f) => get_number_type(f),
        Type::Integer(f) => get_integer_type(f),
        Type::Object(_) => "serde_json::Map<String, serde_json::Value>".into(),
        Type::Array(a) => gen_array_type(a),
        Type::Boolean {} => "bool".into(),
    }
}

fn gen_property_type_for_schema_kind(sk: SchemaKind) -> String {
    let t = match sk {
        SchemaKind::Type(r#type) => r#type,
        _ => panic!("Does not support 'oneOf', 'anyOf' 'allOf', 'not' and 'any'"),
    };
    gen_type_name_for_type(t)
}

fn get_property_type_from_schema_refor(refor: ReferenceOr<Schema>, is_required: bool) -> String {
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
        get_property_type_from_schema_refor(items.unbox(), true)
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

fn generate_struct(
    scope: &mut Scope,
    name: String,
    r#type: Type,
    derivatives: Option<&[&str]>,
    container_attrs: Option<&[(&str, Option<&[&str]>)]>,
    field_attrs: Option<&[(&str, Option<&[&str]>)]>,
) {
    match r#type {
        Type::Object(obj) => {
            let mut derivs = vec!["Debug"];
            if let Some(derivatives) = derivatives {
                derivs.extend(derivatives);
            }

            // add struct
            let r#struct = scope.new_struct(&name).vis("pub");

            // add derives to struct
            for r#derive in derivs {
                r#struct.derive(r#derive);
            }

            // add container attribute
            if let Some(attrs) = container_attrs {
                for (attr, exceptions) in attrs {
                    let is_exception = if let Some(exceptions) = exceptions {
                        exceptions.iter().any(|e| **e == *name.as_str())
                    } else {
                        false
                    };
                    println!("{}", is_exception);
                    if !is_exception {
                        r#struct.attr(*attr);
                    }
                }
            }

            let required = obj.required.into_iter().collect::<HashSet<String>>();
            for (name, refor) in obj.properties {
                let is_required = required.contains(&name);
                let t = get_property_type_from_schema_refor(refor.unbox(), is_required);
                let mut r#field = Field::new(&name.clone().to_snek_case().into_safe(), &t)
                    .vis("pub")
                    .to_owned();
                if let Some(attrs) = field_attrs {
                    for (attr, exceptions) in attrs {
                        let is_exception = if let Some(exceptions) = exceptions {
                            exceptions.iter().any(|e| **e == *name.as_str())
                        } else {
                            false
                        };
                        println!("{}", is_exception);
                        if !is_exception {
                            r#field.annotation(*attr);
                        }
                    }
                }
                r#struct.push_field(field);
            }
        }
        Type::Array(a) => {
            scope.raw(&format!("pub type {} = {};", name, gen_array_type(a)));
        }
        t => {
            println!("#{}: {:#?}", name, t);
            scope.raw(&format!(
                "pub type {} = {};",
                name,
                gen_type_name_for_type(t)
            ));
        }
    }
}

fn generate_enum(
    scope: &mut Scope,
    name: String,
    types: Vec<ReferenceOr<Schema>>,
    derivatives: Option<&[&str]>,
    container_attrs: Option<&[(&str, Option<&[&str]>)]>,
    field_attrs: Option<&[(&str, Option<&[&str]>)]>,
) {
    if let Some(annotations) = container_attrs {
        for (annotation, exceptions) in annotations {
            let is_exception = if let Some(exceptions) = exceptions {
                exceptions.iter().any(|e| **e == *name.as_str())
            } else {
                false
            };
            if !is_exception {
                scope.raw(*annotation);
            }
        }
    }

    let mut derivs = vec!["Debug"];
    if let Some(derivatives) = derivatives {
        derivs.extend(derivatives);
    }

    if let Some(annotations) = field_attrs {
        for (annotation, exceptions) in annotations {
            let is_exception = if let Some(exceptions) = exceptions {
                exceptions.iter().any(|e| **e == *name.as_str())
            } else {
                false
            };
            if !is_exception {
                scope.raw(*annotation);
            }
        }
    }
    let r#enum = scope.new_enum(&name).vis("pub");

    for t in types.into_iter() {
        let t = get_property_type_from_schema_refor(t, true);
        r#enum.new_variant(&t).tuple(&t);
    }
}
