use super::json_asserts::parse_json_response_body_asserts;
use crate::response::common_asserts::{assert_status_less_than, parse_string_asserts};
use std::vec;

use hurl_core::ast::{
    Assert, LineTerminator, Response, Section, Status, Version, VersionValue, Whitespace,
};
use log::{trace, warn};
use oas3::{
    spec::{Operation, RefError},
    Schema, Spec,
};

use crate::{
    content_type::ContentType,
    custom_hurl_ast::{empty_source_info, empty_space, newline},
};

pub enum HandleUnionsBy {
    IgnoringThem,
    TreatingOptionalsAsRequired,
}

pub fn validate_response_not_error() -> Response {
    response_structure(vec![Section {
        line_terminators: vec![],
        space0: empty_space(),
        line_terminator0: LineTerminator {
            space0: empty_space(),
            comment: None,
            newline: empty_space(),
        },
        value: hurl_core::ast::SectionValue::Asserts(vec![assert_status_less_than(400)]),
        source_info: empty_source_info(),
    }])
}

pub fn validation_response_full(
    operation: &Operation,
    spec: &Spec,
    content_type: &ContentType,
    handle_unions_by: HandleUnionsBy,
) -> Result<Option<Response>, RefError> {
    let operation_id = operation
        .operation_id
        .clone()
        .unwrap_or("operationWithNoId".to_string());

    let response = match operation.responses.iter().find(|kv| kv.0 == "200") {
        Some(r) => r.1.resolve(spec)?,
        None => return Ok(Some(validate_response_not_error())),
    };

    let content = match response
        .content
        .iter()
        .find(|c| content_type.matches_string(c.0))
    {
        Some(c) => c,
        None => match response
            .content
            .iter()
            .find(|c| ContentType::is_supported(c.0))
        {
            Some(backup_content) => {
                warn!("operation {operation_id} does not have content type {} defaulting to content type {}", content_type.to_str(), backup_content.0);
                backup_content
            }
            None => {
                warn!("operation {operation_id} does not have any of the supported content types ({}). Defaulting to an empty response body", ContentType::supported_types().join(", "));
                return Ok(None);
            }
        },
    };

    let content_type = match ContentType::from_string(content.0) {
        Ok(ct) => ct,
        Err(_) => {
            warn!("operation {operation_id} does not have any of the supported content types ({}). Defaulting to an empty request body", ContentType::supported_types().join(", "));
            return Ok(None);
        }
    };

    let schema = match &content.1.schema {
        Some(s) => s.resolve(spec)?,
        None => return Ok(None),
    };

    match content_type {
        ContentType::Text => Ok(Some(response_structure(vec![Section {
            line_terminators: vec![],
            space0: empty_space(),
            line_terminator0: newline(),
            value: hurl_core::ast::SectionValue::Asserts(parse_plain_text_response_body(schema)?),
            source_info: empty_source_info(),
        }]))),
        ContentType::Json => Ok(Some(response_structure(vec![Section {
            line_terminators: vec![],
            space0: empty_space(),
            line_terminator0: newline(),
            value: hurl_core::ast::SectionValue::Asserts(parse_json_response_body_asserts(
                schema, &spec, handle_unions_by
            )?),
            source_info: empty_source_info(),
        }]))),
    }
}

fn parse_plain_text_response_body(schema: Schema) -> Result<Vec<Assert>, RefError> {
    trace!("parsing plain text request body");
    let asserts = vec![
        vec![assert_status_less_than(400)],
        parse_string_asserts(schema, &hurl_core::ast::QueryValue::Body),
    ]
    .concat();

    Ok(asserts)
}

fn single_space() -> Whitespace {
    Whitespace {
        value: " ".to_string(),
        source_info: empty_source_info(),
    }
}

fn response_structure(sections: Vec<Section>) -> Response {
    Response {
        line_terminators: vec![newline()],
        version: Version {
            value: VersionValue::VersionAny,
            source_info: empty_source_info(),
        },
        space0: empty_space(),
        status: Status {
            value: hurl_core::ast::StatusValue::Any,
            source_info: empty_source_info(),
        },
        space1: single_space(),
        line_terminator0: newline(),
        headers: vec![],
        sections,
        body: None,
        source_info: empty_source_info(),
    }
}
