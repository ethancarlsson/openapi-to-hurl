use super::json_request_body::parse_json_from_schema;
use hurl_core::ast::Body;
use oas3::{
    spec::{RefError, RequestBody},
    Spec,
};

use crate::{
    cli::{Formatting, Settings},
    custom_hurl_ast::{empty_space, newline},
};

use super::body::parse_schema;

pub struct SpecBodySettings {
    pub formatting: Formatting,
}

impl SpecBodySettings {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            formatting: settings.formatting.clone(),
        }
    }
}

pub fn from_spec_body(
    spec_body: RequestBody,
    spec: &Spec,
    settings: SpecBodySettings,
) -> Result<Option<Body>, RefError> {
    for content in spec_body.content {
        let schema = match parse_schema(content.1.schema, spec)? {
            Some(s) => s,
            None => continue,
        };

        // TODO: implement support for other types and choose types
        if content.0.to_lowercase().contains("json") {
            return match parse_json_from_schema(schema, spec, 1, &settings)? {
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
