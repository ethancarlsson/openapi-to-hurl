use super::json_request_body::parse_json_from_schema;
use hurl_core::ast::{Body, MultilineString, TemplateElement};
use log::{trace, warn};
use oas3::{
    spec::{RefError, RequestBody},
    Schema, Spec,
};

use crate::{
    cli::{Formatting, Settings},
    content_type::ContentType,
    custom_hurl_ast::{empty_source_info, empty_space, newline},
};

use super::body::parse_schema;

pub struct SpecBodySettings {
    pub formatting: Formatting,
    pub content_type: ContentType,
}

impl SpecBodySettings {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            formatting: settings.formatting.clone(),
            content_type: settings.content_type.clone(),
        }
    }
}

pub fn from_spec_body(
    spec_body: RequestBody,
    spec: &Spec,
    operation_id: String,
    settings: SpecBodySettings,
) -> Result<Option<Body>, RefError> {
    let content = match spec_body
        .content
        .iter()
        .find(|c| settings.content_type.matches_string(c.0))
    {
        Some(c) => c,
        None => match spec_body
            .content
            .iter()
            .find(|c| ContentType::is_supported(c.0))
        {
            Some(backup_content) => {
                warn!("operation {operation_id} does not have content type {} defaulting to content type {}", settings.content_type.to_str(), backup_content.0);
                backup_content
            }
            None => {
                warn!("operation {operation_id} does not have any of the supported content types ({}). Defaulting to an empty request body", ContentType::supported_types().join(", "));
                return Ok(None);
            }
        },
    };

    let schema = match parse_schema(content.1.schema.clone(), spec)? {
        Some(s) => s,
        None => return Ok(None),
    };

    let content_type = match ContentType::from_string(content.0) {
        Ok(ct) => ct,
        Err(_) => {
            warn!("operation {operation_id} does not have any of the supported content types ({}). Defaulting to an empty request body", ContentType::supported_types().join(", "));
            return Ok(None);
        }
    };

    match content_type {
        ContentType::Json => {
            trace!("parsing JSON request body");
            match parse_json_from_schema(schema, spec, 1, &settings)? {
                Some(v) => Ok(Some(Body {
                    line_terminators: vec![],
                    space0: empty_space(),
                    value: hurl_core::ast::Bytes::Json(v),
                    line_terminator0: newline(),
                })),
                None => Ok(None),
            }
        }
        ContentType::PlainText => match parse_plain_text(schema)? {
            Some(v) => Ok(Some(Body {
                line_terminators: vec![],
                space0: empty_space(),
                value: hurl_core::ast::Bytes::MultilineString(v),
                line_terminator0: newline(),
            })),
            None => Ok(None),
        },
    }
}

fn parse_plain_text(schema: Schema) -> Result<Option<hurl_core::ast::MultilineString>, RefError> {
    trace!("parsing plain text request body");
    if schema.read_only.unwrap_or(false) {
        return Ok(None);
    }

    match schema.example {
        Some(example) => match example {
            _ => Ok(Some(MultilineString::Text(hurl_core::ast::Text {
                space: empty_space(),
                newline: empty_space(),
                value: hurl_core::ast::Template {
                    delimiter: None,
                    source_info: empty_source_info(),
                    elements: vec![TemplateElement::String {
                        value: "".to_string(),
                        // It thinks it's json so it adds unnecessary " characters around strings
                        encoded: example.to_string().trim_matches('"').to_string(),
                    }],
                },
            }))),
        },
        // If there's no example we can't tell the structure just give an empty value
        None => Ok(Some(MultilineString::Text(hurl_core::ast::Text {
            space: empty_space(),
            newline: empty_space(),
            value: hurl_core::ast::Template {
                delimiter: None,
                source_info: empty_source_info(),
                elements: vec![TemplateElement::String {
                    value: "".to_string(),
                    encoded: "".to_string(),
                }],
            },
        }))),
    }
}
