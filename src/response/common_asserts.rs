use hurl_core::ast::{
    Assert, Comment, LineTerminator, Predicate, PredicateFunc, PredicateFuncValue, PredicateValue,
    Query, QueryValue, Whitespace, 
};
use oas3::Schema;
use regex::Regex;

use crate::custom_hurl_ast::{empty_source_info, empty_space};

pub fn assert_query_matches_predicate(
    query: &hurl_core::ast::QueryValue,
    predicate: PredicateFuncValue,
) -> Assert {
    assert_query_matches_predicate_with_filters(query, predicate, vec![], None)
}

pub fn parse_string_asserts(schema: Schema, query_value: &QueryValue) -> Vec<Assert> {
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

    add_string_schema_asserts_to_asserts(schema, query_value, asserts)
}

fn add_string_schema_asserts_to_asserts(
    schema: Schema,
    query_value: &QueryValue,
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
                    value: PredicateValue::Regex(regex_from_pattern(format!("^.{{{}}}", min + 1))),
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

pub fn assert_query_matches_with_comment(
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

pub fn assert_query_matches_predicate_with_filters(
    query: &hurl_core::ast::QueryValue,
    predicate: PredicateFuncValue,
    filters: Vec<hurl_core::ast::Filter>,
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

pub fn assert_status_less_than(num: i64) -> Assert { assert_query_matches_predicate(
        &hurl_core::ast::QueryValue::Status,
        PredicateFuncValue::LessThan {
            space0: single_space(),
            value: hurl_core::ast::PredicateValue::Integer(num),
            operator: true,
        },
    )
}


fn single_space() -> Whitespace {
    Whitespace {
        value: " ".to_string(),
        source_info: empty_source_info(),
    }
}

pub fn regex_from_pattern(pattern: String) -> hurl_core::ast::Regex {
    hurl_core::ast::Regex {
        inner: Regex::new(&pattern).unwrap(),
    }
}
