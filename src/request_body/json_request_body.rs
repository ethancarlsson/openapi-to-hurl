use log::debug;
use oas3::{
    spec::{ObjectOrReference, RefError},
    Schema, Spec,
};
use serde_json::{Map, Number};

use super::request_body::SpecBodySettings;

pub fn parse_json_from_schema(
    schema: Schema,
    spec: &Spec,
    settings: &SpecBodySettings,
) -> Result<Option<serde_json::Value>, RefError> {
    if schema.read_only.unwrap_or(false) {
        return Ok(None);
    }

    match schema.example {
        Some(ex) => return Ok(Some(ex)),
        None => (),
    }

    match schema.enum_values.first() {
        Some(e) => return Ok(Some(serde_json::Value::String(e.to_string()))),
        None => (),
    }

    if schema.all_of.len() > 0 {
        return Ok(Some(json_obj_from_allof(schema, spec, settings)?));
    }

    if schema.one_of.len() > 0 {
        return Ok(json_obj_from_anyof(schema.one_of, spec, &settings)?);
    }

    // Treat any_of and one_of the same / use only the first schema of both
    if schema.any_of.len() > 0 {
        return Ok(json_obj_from_anyof(schema.any_of, spec, &settings)?);
    }

    let default_val = match schema.schema_type {
        Some(ref typeset) => match typeset {
            oas3::spec::SchemaTypeSet::Single(t) => Some(default_json_value_from_schema_type(*t)),
            oas3::spec::SchemaTypeSet::Multiple(ts) => match ts.first() {
                Some(t) => Some(default_json_value_from_schema_type(*t)),
                None => None,
            },
        },
        None => None,
    };

    match default_val {
        Some(v) => {
            return match v {
                SimpleJsonValue::Scalar(s) => Ok(Some(s)),
                SimpleJsonValue::Array => match schema.items {
                    Some(items_schema) => {
                        let schema = match items_schema.resolve(spec) {
                            Ok(s) => parse_json_from_schema(s, spec, settings)?,
                            Err(e) => return Err(e),
                        };

                        match schema {
                            Some(s) => Ok(Some(serde_json::Value::Array(vec![s]))),
                            None => Ok(Some(serde_json::Value::Array(vec![]))),
                        }
                    }
                    None => Ok(Some(serde_json::Value::Array(vec![]))),
                },
                SimpleJsonValue::Object => {
                    let props = parse_json_object(&schema, spec, settings)?;

                    Ok(Some(serde_json::Value::Object(props)))
                }
            };
        }
        None => {
            if schema.properties.len() > 0 {
                let props = parse_json_object(&schema, spec, settings)?;
                Ok(Some(serde_json::Value::Object(props)))
            } else if schema.items.is_some() {
                let items_schema = match schema.items.unwrap().resolve(spec) {
                    Ok(s) => parse_json_from_schema(s, spec, settings)?,
                    Err(e) => return Err(e),
                };

                match items_schema {
                    Some(s) => Ok(Some(serde_json::Value::Array(vec![s]))),
                    None => Ok(Some(serde_json::Value::Array(vec![]))),
                }
            } else {
                debug!("Couldn't build anything from schema. Returning null...");

                Ok(Some(serde_json::Value::Null))
            }
        }
    }
}

fn parse_json_object(
    schema: &Schema,
    spec: &Spec,
    settings: &SpecBodySettings,
) -> Result<Map<String, serde_json::Value>, RefError> {
    let mut props = Map::new();
    for prop in &schema.properties {
        let val = parse_json_from_schema(prop.1.resolve(spec)?, spec, settings)?;

        match val {
            Some(v) => props.insert(prop.0.to_string(), v),
            None => None,
        };
    }
    Ok(props)
}

enum SimpleJsonValue {
    Scalar(serde_json::Value),
    Array,
    Object,
}

fn default_json_value_from_schema_type(schema_type: oas3::spec::SchemaType) -> SimpleJsonValue {
    match schema_type {
        oas3::spec::SchemaType::Boolean => SimpleJsonValue::Scalar(serde_json::Value::Bool(true)),
        oas3::spec::SchemaType::Integer => {
            SimpleJsonValue::Scalar(serde_json::Value::Number(Number::from(3)))
        }
        oas3::spec::SchemaType::Number => {
            // Safe to unwrap, only returns None for infinite values or for NaN
            SimpleJsonValue::Scalar(serde_json::Value::Number(Number::from_f64(3.3).unwrap()))
        }
        oas3::spec::SchemaType::String => {
            SimpleJsonValue::Scalar(serde_json::Value::String("string".to_string()))
        }
        oas3::spec::SchemaType::Array => SimpleJsonValue::Array,
        oas3::spec::SchemaType::Object => SimpleJsonValue::Object,
        oas3::spec::SchemaType::Null => SimpleJsonValue::Scalar(serde_json::Value::Null),
    }
}

fn json_obj_from_anyof(
    anyof: Vec<ObjectOrReference<Schema>>,
    spec: &Spec,
    settings: &SpecBodySettings,
) -> Result<Option<serde_json::Value>, RefError> {
    for schema in &anyof {
        return parse_json_from_schema(schema.resolve(spec)?, spec, &settings);
    }

    Ok(Some(serde_json::Value::Object(Map::new())))
}

fn json_obj_from_allof(
    schema: Schema,
    spec: &Spec,
    settings: &SpecBodySettings,
) -> Result<serde_json::Value, RefError> {
    let mut props = Map::new();
    for subschema in schema.all_of {
        for prop in subschema.resolve(spec)?.properties {
            let value = parse_json_from_schema(prop.1.resolve(spec)?, spec, &settings)?;
            match value {
                Some(v) => {
                    props.insert(prop.0, v);
                }
                None => (),
            };
        }
    }

    for prop in schema.properties {
        let value = parse_json_from_schema(prop.1.resolve(spec)?, spec, &settings)?;
        match value {
            Some(v) => {
                props.insert(prop.0, v);
            }
            None => (),
        };
    }

    Ok(serde_json::Value::Object(props))
}
