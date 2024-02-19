use super::json_request_body::parse_json_from_schema;
use crate::content_type::ContentType;
use crate::Settings;
use crate::{
    cli::Formatting,
    custom_hurl_ast::{empty_source_info, empty_space, newline},
};
use anyhow::Context;
use hurl_core::ast::{Body, MultilineString, TemplateElement};
use log::{debug, trace, warn};
use oas3::{
    spec::{RefError, RequestBody},
    Schema, Spec,
};

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

    let schema = match &content.1.schema {
        Some(s) => s.resolve(spec)?,
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
            match parse_json_from_schema(schema, spec, &settings)? {
                Some(v) => match to_json_string(&v, settings) {
                    Ok(inner_json) => Ok(Some(Body {
                        line_terminators: vec![],
                        space0: empty_space(),
                        value: hurl_core::ast::Bytes::MultilineString(MultilineString::Json(text(
                            inner_json,
                        ))),
                        line_terminator0: newline(),
                    })),
                    Err(e) => {
                        // There's no real reason this should happen.
                        debug!("Could not transform the specification for {operation_id} to JSON {e}. Defaulting to empty request body");

                        Ok(None)
                    }
                },
                None => Ok(None),
            }
        }
        ContentType::Text => match parse_plain_text(schema)? {
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
            e => Ok(Some(MultilineString::Text(hurl_core::ast::Text {
                space: empty_space(),
                newline: empty_space(),
                value: hurl_core::ast::Template {
                    delimiter: Some('\n'),
                    source_info: empty_source_info(),
                    elements: vec![TemplateElement::String {
                        value: "".to_string(),
                        // It thinks it's json so it adds unnecessary " characters around strings
                        encoded: rem_first_and_last(e.to_string()),
                    }],
                },
            }))),
        },
        // If there's no example we can't tell the structure just give an empty value
        None => Ok(Some(MultilineString::Text(text("".to_string())))),
    }
}

// https://stackoverflow.com/questions/65976432/how-to-remove-first-and-last-character-of-a-string-in-rust
fn rem_first_and_last(value: String) -> String {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str().to_string()
}

fn text(element: String) -> hurl_core::ast::Text {
    hurl_core::ast::Text {
        space: empty_space(),
        newline: empty_space(),
        value: hurl_core::ast::Template {
            delimiter: Some('\n'),
            source_info: empty_source_info(),
            elements: vec![TemplateElement::String {
                value: "".to_string(),
                encoded: element,
            }],
        },
    }
}

fn to_json_string(
    json_value: &serde_json::Value,
    settings: SpecBodySettings,
) -> Result<String, anyhow::Error> {
    match settings.formatting {
        Formatting::NoFormatting => {
            serde_json::to_string(json_value).with_context(|| "couldn't serialize to string")
        }
        Formatting::RequestBodies => serde_json::to_string_pretty(json_value)
            .with_context(|| "couldn't serialize to pretty string"),
    }
}
