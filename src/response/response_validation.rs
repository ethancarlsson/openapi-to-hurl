use std::{i64, vec};

use hurl_core::ast::{
    Assert, Comment, Filter, LineTerminator, Predicate, PredicateFunc, PredicateFuncValue,
    PredicateValue, Query, QueryValue, Response, Section, Status, Version, VersionValue,
    Whitespace,
};
use log::{trace, warn};
use oas3::{
    spec::{Operation, RefError},
    Schema, Spec,
};
use regex::Regex;

use crate::{
    content_type::ContentType,
    custom_hurl_ast::{empty_source_info, empty_space, newline},
    schema::schema::parse_schema,
};

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
) -> Result<Option<Response>, RefError> {
    let operation_id = operation
        .operation_id
        .clone()
        .unwrap_or("operationWithNoId".to_string());

    let response = match operation.responses(spec).iter().find(|kv| kv.0 == "200") {
        Some(r) => r.1.clone(),
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
                warn!("operation {operation_id} does not have any of the supported content types ({}). Defaulting to an empty request body", ContentType::supported_types().join(", "));
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

    let schema = match parse_schema(content.1.schema.clone(), spec)? {
        Some(s) => s,
        None => return Ok(None),
    };

    match content_type {
        ContentType::Text => Ok(Some(response_structure(vec![Section {
            line_terminators: vec![],
            space0: empty_space(),
            line_terminator0: LineTerminator {
                space0: empty_space(),
                comment: None,
                newline: empty_space(),
            },
            value: hurl_core::ast::SectionValue::Asserts(parse_plain_text_response_body(schema)?),
            source_info: empty_source_info(),
        }]))),
        ContentType::Json => todo!(),
    }
}

fn parse_plain_text_response_body(schema: Schema) -> Result<Vec<Assert>, RefError> {
    trace!("parsing plain text request body");
    let asserts = vec![
        vec![assert_status_less_than(400)],
        parse_string_asserts(schema, hurl_core::ast::QueryValue::Body),
    ]
    .concat();

    Ok(asserts)
}

fn parse_string_asserts(schema: Schema, query_value: QueryValue) -> Vec<Assert> {
    match schema.write_only {
        Some(is_write_only) => {
            if is_write_only {
                return vec![];
            }
        }
        None => (),
    }

    let asserts = vec![assert_query_matches_predicate(
        &query_value,
        PredicateFuncValue::IsString,
    )];

    add_common_schema_asserts_to_asserts(schema, query_value, asserts)
}

fn add_common_schema_asserts_to_asserts(
    schema: Schema,
    query_value: QueryValue,
    mut asserts: Vec<Assert>,
) -> Vec<Assert> {
    match schema.pattern {
        Some(p) => asserts.push(assert_query_matches_predicate(
            &query_value,
            PredicateFuncValue::Match {
                space0: single_space(),
                value: PredicateValue::Regex(regex_from_pattern(p)),
            },
        )),
        None => (),
    };

    match schema.min_length {
        Some(min) => asserts.push(assert_query_matches_with_comment(
            &query_value,
            if schema.exclusive_minimum == Some(true) {
                PredicateFuncValue::Match {
                    space0: single_space(),
                    value: PredicateValue::Regex(regex_from_pattern(format!(
                        "^.{{{}}}",
                        min + 1
                    ))),
                }
            } else {
                PredicateFuncValue::Match {
                    space0: single_space(),
                    value: PredicateValue::Regex(regex_from_pattern(format!("^.{{0,{min}}}$"))),
                }
            },
            "assert max length".to_string(),
        )),
        None => (),
    };

    match schema.max_length {
        Some(max) => asserts.push(assert_query_matches_with_comment(
            &query_value,
            if schema.exclusive_maximum == Some(true) {
                PredicateFuncValue::Match {
                    space0: single_space(),
                    value: PredicateValue::Regex(regex_from_pattern(format!(
                        "^.{{0,{}}}$",
                        max - 1
                    ))),
                }
            } else {
                PredicateFuncValue::Match {
                    space0: single_space(),
                    value: PredicateValue::Regex(regex_from_pattern(format!("^.{{0,{max}}}$"))),
                }
            },
            "assert max length".to_string(),
        )),
        None => (),
    };

    asserts
}

fn regex_from_pattern(pattern: String) -> hurl_core::ast::Regex {
    hurl_core::ast::Regex {
        inner: Regex::new(&pattern).unwrap(),
    }
}

fn single_space() -> Whitespace {
    Whitespace {
        value: " ".to_string(),
        source_info: empty_source_info(),
    }
}

fn assert_status_less_than(num: i64) -> Assert {
    assert_query_matches_predicate(
        &hurl_core::ast::QueryValue::Status,
        PredicateFuncValue::LessThan {
            space0: single_space(),
            value: hurl_core::ast::PredicateValue::Integer(num),
            operator: true,
        },
    )
}

fn assert_query_matches_predicate(
    query: &hurl_core::ast::QueryValue,
    predicate: PredicateFuncValue,
) -> Assert {
    assert_query_matches_predicate_with_filters(query, predicate, vec![], None)
}

fn assert_query_matches_with_comment(
    query: &hurl_core::ast::QueryValue,
    predicate: PredicateFuncValue,
    comment: String,
) -> Assert {
    assert_query_matches_predicate_with_filters(
        query,
        predicate,
        vec![],
        Some(Comment { value: comment }),
    )
}

fn assert_query_matches_predicate_with_filters(
    query: &hurl_core::ast::QueryValue,
    predicate: PredicateFuncValue,
    filters: Vec<Filter>,
    comment: Option<Comment>,
) -> Assert {
    Assert {
        line_terminators: vec![],
        space0: Whitespace {
            value: "\n".to_string(),
            source_info: empty_source_info(),
        },
        query: Query {
            source_info: empty_source_info(),
            value: query.clone(),
        },
        filters: filters
            .iter()
            .map(|filter| (single_space(), filter.clone()))
            .collect(),
        space1: single_space(),
        predicate: Predicate {
            not: false,
            space0: single_space(),
            predicate_func: PredicateFunc {
                source_info: empty_source_info(),
                value: predicate,
            },
        },
        line_terminator0: LineTerminator {
            space0: match comment {
                Some(_) => single_space(),
                None => empty_space(),
            },
            comment,
            newline: Whitespace {
                value: "".to_string(),
                source_info: empty_source_info(),
            },
        },
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
