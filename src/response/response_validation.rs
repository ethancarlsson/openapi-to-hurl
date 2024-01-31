use hurl_core::ast::{
    Assert, LineTerminator, Predicate, PredicateFunc, PredicateFuncValue, Query, Response, Section,
    Status, Version, VersionValue, Whitespace,
};

use crate::custom_hurl_ast::{empty_source_info, empty_space, newline};

fn single_space() -> Whitespace {
    Whitespace {
        value: " ".to_string(),
        source_info: empty_source_info(),
    }
}

pub fn validate_response_not_error() -> Response {
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
        sections: vec![Section {
            line_terminators: vec![],
            space0: empty_space(),
            line_terminator0: LineTerminator {
                space0: empty_space(),
                comment: None,
                newline: empty_space(),
            },
            value: hurl_core::ast::SectionValue::Asserts(vec![Assert {
                line_terminators: vec![newline()],
                space0: empty_space(),
                query: Query {
                    source_info: empty_source_info(),
                    value: hurl_core::ast::QueryValue::Status,
                },
                filters: vec![],
                space1: single_space(),
                predicate: Predicate {
                    not: false,
                    space0: single_space(),
                    predicate_func: PredicateFunc {
                        source_info: empty_source_info(),
                        value: PredicateFuncValue::LessThan {
                            space0: single_space(),
                            value: hurl_core::ast::PredicateValue::Integer(400),
                            operator: true,
                        },
                    },
                },
                line_terminator0: newline(),
            }]),
            source_info: empty_source_info(),
        }],
        body: None,
        source_info: empty_source_info(),
    }
}
