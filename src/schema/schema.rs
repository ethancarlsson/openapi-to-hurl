use log::trace;
use oas3::{
    spec::{FromRef, ObjectOrReference, RefError},
    Schema, Spec,
};
use serde_json::json;

pub fn parse_schema(
    schema: Option<ObjectOrReference<Schema>>,
    spec: &Spec,
) -> Result<Option<Schema>, RefError> {
    match schema {
        Some(s) => match s {
            ObjectOrReference::Object(s) => Ok(Some(s)),
            ObjectOrReference::Ref { ref_path } => match Schema::from_ref(&spec, &ref_path) {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(e),
            },
        },

        None => Ok(None),
    }
}

pub trait SchemaVisitor<T> {
    fn new() -> Self;
    fn add_value(&mut self, val: serde_json::Value) -> &Self;
    fn get_value(&self) -> &T;
}

pub fn parse_plain_text_schema<T>(
    schema: Schema,
    visitor: &mut impl SchemaVisitor<T>,
) -> Result<Option<&impl SchemaVisitor<T>>, RefError> {
    trace!("parsing plain text request body");
    if schema.read_only.unwrap_or(false) {
        return Ok(None);
    }

    match schema.example {
        Some(example) => match example {
            e => Ok(Some(visitor.add_value(e))),
        },
        // If there's no example we can't tell the structure just give an empty value
        None => Ok(Some(visitor.add_value(json!("")))),
    }
}
