use hurl_core::ast::{Body, JsonListElement};
use log::debug;
use oas3::{
    spec::{ObjectOrReference, RefError, RequestBody},
    Schema, Spec,
};

use crate::custom_hurl_ast::{empty_space, newline};

use super::body::{parse_schema, template_from_string};

pub fn from_spec_body(spec_body: RequestBody, spec: &Spec) -> Result<Option<Body>, RefError> {
    for content in spec_body.content {
        let schema = match parse_schema(content.1.schema, spec)? {
            Some(s) => s,
            None => continue,
        };

        // TODO: implement support for other types and choose types
        if content.0.to_lowercase().contains("json") {
            return match parse_json_from_schema(schema, spec, 1)? {
                Some(v) => Ok(Some(Body {
                    line_terminators: vec![],
                    space0: empty_space(),
                    value: hurl_core::ast::Bytes::Json(v),
                    line_terminator0: newline(),
                })),
                None => Ok(None),
            };
        }
    }

    Ok(None)
}

fn serde_to_hurl_json(serde_val: &serde_json::Value) -> hurl_core::ast::JsonValue {
    match serde_val {
        serde_json::Value::Null => hurl_core::ast::JsonValue::Null,
        serde_json::Value::Bool(b) => hurl_core::ast::JsonValue::Boolean(*b),
        serde_json::Value::Number(n) => hurl_core::ast::JsonValue::Number(n.to_string()),
        serde_json::Value::String(s) => hurl_core::ast::JsonValue::String(template_from_string(&s)),
        serde_json::Value::Array(ref arr) => hurl_core::ast::JsonValue::List {
            space0: " ".to_string(),
            elements: arr
                .iter()
                .map(|el| JsonListElement {
                    space0: " ".to_string(),
                    value: serde_to_hurl_json(el),
                    space1: " ".to_string(),
                })
                .collect(),
        },
        serde_json::Value::Object(o) => hurl_core::ast::JsonValue::Object {
            space0: "".to_string(),
            elements: o
                .into_iter()
                .map(|prop| hurl_core::ast::JsonObjectElement {
                    space0: "\n  ".to_string(),
                    name: template_from_string(&prop.0),
                    space1: " ".to_string(),
                    space2: " ".to_string(),
                    value: serde_to_hurl_json(prop.1),
                    space3: "".to_string(),
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

fn parse_json_from_schema(
    schema: Schema,
    spec: &Spec,
    depth: usize,
) -> Result<Option<hurl_core::ast::JsonValue>, RefError> {
    if schema.read_only.unwrap_or(false) {
        return Ok(None);
    }

    match schema.example {
        Some(ex) => return Ok(Some(serde_to_hurl_json(&ex))),
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
                            Ok(s) => parse_json_from_schema(s, spec, depth)?,
                            Err(e) => return Err(e),
                        };

                        Ok(Some(hurl_core::ast::JsonValue::List {
                            space0: " ".to_string(),
                            elements: match schema {
                                Some(s) => vec![JsonListElement {
                                    space0: " ".to_string(),
                                    value: s,
                                    space1: " ".to_string(),
                                }],
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
                        let val = parse_json_from_schema(prop.1.resolve(spec)?, spec, depth + 1)?;
                        match val {
                            Some(v) => props.push(hurl_core::ast::JsonObjectElement {
                                space0: "\n".to_string() + &"  ".repeat(depth),
                                name: template_from_string(&prop.0),
                                space1: "".to_string(),
                                space2: " ".to_string(),
                                value: v,
                                space3: "".to_string(),
                            }),
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
                return Ok(Some(json_obj_from_allof(schema.all_of, spec, depth)?));
            }

            if schema.one_of.len() > 0 {
                return Ok(json_obj_from_anyof(schema.one_of, spec, depth)?);
            }

            // Treat any_of and one_of the same / use only the first schema of both
            if schema.any_of.len() > 0 {
                return Ok(json_obj_from_anyof(schema.any_of, spec, depth)?);
            }

            debug!("Couldn't build anything from schema. Returning null...");

            Ok(Some(hurl_core::ast::JsonValue::Null))
        }
    }
}

fn json_obj_from_anyof(
    anyof: Vec<ObjectOrReference<Schema>>,
    spec: &Spec,
    depth: usize,
) -> Result<Option<hurl_core::ast::JsonValue>, RefError> {
    for schema in &anyof {
        return parse_json_from_schema(schema.resolve(spec)?, spec, depth);
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
) -> Result<hurl_core::ast::JsonValue, RefError> {
    let mut properties = vec![];
    for schema in allof {
        for prop in schema.resolve(spec)?.properties {
            let value = parse_json_from_schema(prop.1.resolve(spec)?, spec, depth + 1)?;
            match value {
                Some(v) => properties.push(hurl_core::ast::JsonObjectElement {
                    space0: "\n".to_string() + &"  ".repeat(depth),
                    name: template_from_string(&prop.0),
                    space1: "".to_string(),
                    space2: " ".to_string(),
                    value: v,
                    space3: "".to_string(),
                }),
                None => (),
            }
        }
    }

    Ok(hurl_core::ast::JsonValue::Object {
        space0: "".to_string(),
        elements: properties,
    })
}
