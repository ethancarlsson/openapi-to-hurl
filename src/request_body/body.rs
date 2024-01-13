use hurl_core::ast::{Template, TemplateElement};
use oas3::{
    spec::{FromRef, ObjectOrReference, RefError},
    Schema, Spec,
};

use crate::custom_hurl_ast::empty_source_info;

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

pub fn template_from_string(s: &String) -> Template {
    Template {
        delimiter: Some('"'),
        elements: vec![TemplateElement::String {
            value: s.to_string(),
            encoded: s.to_string(),
        }],
        source_info: empty_source_info(),
    }
}
