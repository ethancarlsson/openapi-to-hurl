use hurl_core::ast::{JsonValue, Template};
use log::debug;
use oas3::{
    spec::{ObjectOrReference, RefError},
    Schema, Spec,
};

use crate::cli::Formatting;

use super::{body::template_from_string, request_body::SpecBodySettings};

pub fn parse_json_from_schema(
    schema: Schema,
    spec: &Spec,
    depth: usize,
    settings: &SpecBodySettings,
) -> Result<Option<hurl_core::ast::JsonValue>, RefError> {
    if schema.read_only.unwrap_or(false) {
        return Ok(None);
    }

    match schema.example {
        Some(ex) => return Ok(Some(serde_to_hurl_json(&ex, depth, settings))),
        None => (),
    }

    let default_val = match schema.schema_type {
        Some(t) => Some(default_json_value_from_schema_type(t)),
        None => None,
    };

    match default_val {
        Some(v) => {
            return match v {
                SimpleJsonValue::Scalar(s) => Ok(Some(s)),
                SimpleJsonValue::Array => match schema.items {
                    Some(items_schema) => {
                        let schema = match items_schema.resolve(spec) {
                            Ok(s) => parse_json_from_schema(s, spec, depth, settings)?,
                            Err(e) => return Err(e),
                        };

                        Ok(Some(hurl_core::ast::JsonValue::List {
                            space0: build_json_list_space(&settings.formatting),
                            elements: match schema {
                                Some(s) => vec![build_json_list_value(s, &settings.formatting)],
                                None => vec![],
                            },
                        }))
                    }
                    None => Ok(Some(hurl_core::ast::JsonValue::List {
                        space0: "\n".to_string(),
                        elements: vec![],
                    })),
                },
                SimpleJsonValue::Object => {
                    let mut props = vec![];

                    for prop in schema.properties {
                        let val = parse_json_from_schema(
                            prop.1.resolve(spec)?,
                            spec,
                            depth + 1,
                            settings,
                        )?;
                        match val {
                            Some(v) => props.push(build_json_object_element(
                                template_from_string(&prop.0),
                                v,
                                depth,
                                &settings.formatting,
                            )),
                            None => (),
                        }
                    }

                    Ok(Some(hurl_core::ast::JsonValue::Object {
                        space0: "".to_string(),
                        elements: props,
                    }))
                }
            }
        }
        None => {
            if schema.all_of.len() > 0 {
                return Ok(Some(json_obj_from_allof(
                    schema.all_of,
                    spec,
                    depth,
                    settings,
                )?));
            }

            if schema.one_of.len() > 0 {
                return Ok(json_obj_from_anyof(schema.one_of, spec, depth, &settings)?);
            }

            // Treat any_of and one_of the same / use only the first schema of both
            if schema.any_of.len() > 0 {
                return Ok(json_obj_from_anyof(schema.any_of, spec, depth, &settings)?);
            }

            debug!("Couldn't build anything from schema. Returning null...");

            Ok(Some(hurl_core::ast::JsonValue::Null))
        }
    }
}

fn serde_to_hurl_json(
    serde_val: &serde_json::Value,
    depth: usize,
    settings: &SpecBodySettings,
) -> hurl_core::ast::JsonValue {
    match serde_val {
        serde_json::Value::Null => hurl_core::ast::JsonValue::Null,
        serde_json::Value::Bool(b) => hurl_core::ast::JsonValue::Boolean(*b),
        serde_json::Value::Number(n) => hurl_core::ast::JsonValue::Number(n.to_string()),
        serde_json::Value::String(s) => hurl_core::ast::JsonValue::String(template_from_string(&s)),
        serde_json::Value::Array(ref arr) => hurl_core::ast::JsonValue::List {
            space0: build_json_list_space(&settings.formatting),
            elements: arr
                .iter()
                .map(|el| {
                    build_json_list_value(
                        serde_to_hurl_json(el, depth, settings),
                        &settings.formatting,
                    )
                })
                .collect(),
        },
        serde_json::Value::Object(o) => hurl_core::ast::JsonValue::Object {
            space0: "".to_string(),
            elements: o
                .into_iter()
                .map(|prop| {
                    build_json_object_element(
                        template_from_string(&prop.0),
                        serde_to_hurl_json(prop.1, depth, settings),
                        depth,
                        &settings.formatting,
                    )
                })
                .collect(),
        },
    }
}

enum SimpleJsonValue {
    Scalar(hurl_core::ast::JsonValue),
    Array,
    Object,
}

fn default_json_value_from_schema_type(schema_type: oas3::spec::SchemaType) -> SimpleJsonValue {
    match schema_type {
        oas3::spec::SchemaType::Boolean => {
            SimpleJsonValue::Scalar(hurl_core::ast::JsonValue::Boolean(true))
        }
        oas3::spec::SchemaType::Integer => {
            SimpleJsonValue::Scalar(hurl_core::ast::JsonValue::Number(3.to_string()))
        }
        oas3::spec::SchemaType::Number => {
            SimpleJsonValue::Scalar(hurl_core::ast::JsonValue::Number(3.3.to_string()))
        }
        oas3::spec::SchemaType::String => SimpleJsonValue::Scalar(
            hurl_core::ast::JsonValue::String(template_from_string(&"string".to_string())),
        ),
        oas3::spec::SchemaType::Array => SimpleJsonValue::Array,
        oas3::spec::SchemaType::Object => SimpleJsonValue::Object,
    }
}

fn json_obj_from_anyof(
    anyof: Vec<ObjectOrReference<Schema>>,
    spec: &Spec,
    depth: usize,
    settings: &SpecBodySettings,
) -> Result<Option<hurl_core::ast::JsonValue>, RefError> {
    for schema in &anyof {
        return parse_json_from_schema(schema.resolve(spec)?, spec, depth, &settings);
    }

    Ok(Some(hurl_core::ast::JsonValue::Object {
        space0: "".to_string(),
        elements: vec![],
    }))
}

fn json_obj_from_allof(
    allof: Vec<ObjectOrReference<Schema>>,
    spec: &Spec,
    depth: usize,
    settings: &SpecBodySettings,
) -> Result<hurl_core::ast::JsonValue, RefError> {
    let mut properties = vec![];
    for schema in allof {
        for prop in schema.resolve(spec)?.properties {
            let value = parse_json_from_schema(prop.1.resolve(spec)?, spec, depth + 1, &settings)?;
            match value {
                Some(v) => properties.push(build_json_object_element(
                    template_from_string(&prop.0),
                    v,
                    depth,
                    &settings.formatting,
                )),
                None => (),
            }
        }
    }

    Ok(hurl_core::ast::JsonValue::Object {
        space0: "".to_string(),
        elements: properties,
    })
}

fn build_json_object_element(
    name: Template,
    value: JsonValue,
    depth: usize,
    formatting: &Formatting,
) -> hurl_core::ast::JsonObjectElement {
    match formatting {
        Formatting::NoFormatting => hurl_core::ast::JsonObjectElement {
            space0: "".to_string(),
            name,
            space1: "".to_string(),
            space2: "".to_string(),
            value,
            space3: "".to_string(),
        },
        Formatting::RequestBodies => hurl_core::ast::JsonObjectElement {
            space0: "\n".to_string() + &"  ".repeat(depth),
            name,
            space1: "".to_string(),
            space2: " ".to_string(),
            value,
            space3: "".to_string(),
        },
    }
}

fn build_json_list_space(formatting: &Formatting) -> String {
    match formatting {
        Formatting::NoFormatting => "".to_string(),
        Formatting::RequestBodies => " ".to_string(),
    }
}

fn build_json_list_value(
    value: JsonValue,
    formatting: &Formatting,
) -> hurl_core::ast::JsonListElement {
    match formatting {
        Formatting::NoFormatting => hurl_core::ast::JsonListElement {
            space0: "".to_string(),
            value,
            space1: "".to_string(),
        },
        Formatting::RequestBodies => hurl_core::ast::JsonListElement {
            space0: " ".to_string(),
            value,
            space1: " ".to_string(),
        },
    }
}
