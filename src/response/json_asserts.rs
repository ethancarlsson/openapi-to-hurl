use std::collections::BTreeMap;

use hurl_core::ast::{Assert, FilterValue, Float, PredicateValue, TemplateElement};
use log::{debug, warn};
use oas3::{spec::RefError, Schema, Spec};

use crate::{
    custom_hurl_ast::empty_source_info, hurl_files::single_space,
    response::common_asserts::assert_query_matches_predicate,
};

use super::{
    common_asserts::{
        assert_query_matches_predicate_with_filters, assert_status_less_than, parse_string_asserts,
    },
    response_validation::HandleUnionsBy,
};

pub struct SchemaToJsonAssertBuilder<'a> {
    asserts: &'a mut Vec<Assert>,
    spec: &'a Spec,
    handle_unions_by: &'a HandleUnionsBy,
}

impl<'a> SchemaToJsonAssertBuilder<'a> {
    fn new(
        asserts: &'a mut Vec<Assert>,
        spec: &'a Spec,
        handle_unions_by: &'a HandleUnionsBy,
    ) -> Self {
        Self {
            asserts,
            spec,
            handle_unions_by,
        }
    }

    fn add_asserts_from_schema(
        &mut self,
        schema: Schema,
        query_value: &hurl_core::ast::QueryValue,
    ) -> Result<&Self, RefError> {
        match schema.write_only {
            Some(is_write_only) => {
                if is_write_only {
                    return Ok(self);
                }
            }
            None => (),
        }

        if schema.all_of.len() > 0 {
            let combined_schema = self.build_schema_from_allof(schema)?;
            return self.add_asserts_from_schema(combined_schema, query_value);
        }

        let schema_type = match schema.schema_type {
            Some(t) => t,
            None => {
                if schema.properties.len() > 0 {
                    oas3::spec::SchemaType::Object
                } else if schema.items.is_some() {
                    oas3::spec::SchemaType::Array
                } else {
                    return Ok(self);
                }
            }
        };

        // This tool can't handle union types.
        if schema.nullable == Some(true) || !schema.one_of.is_empty() || !schema.any_of.is_empty() {
            debug!("Schema {} is nullable or uses oneOf/anyOf, this tool can't generate assertions for schemas with multiple possible types", schema.title.unwrap_or("".to_string()));
            return Ok(self);
        }

        match schema_type {
            oas3::spec::SchemaType::Boolean => self.asserts.push(assert_query_matches_predicate(
                &query_value,
                hurl_core::ast::PredicateFuncValue::IsBoolean,
            )),
            oas3::spec::SchemaType::Integer => self.add_int_asserts(schema, query_value),
            oas3::spec::SchemaType::Number => self.add_number_asserts(schema, query_value),
            oas3::spec::SchemaType::String => self.add_string_asserts(schema, query_value),
            oas3::spec::SchemaType::Array => self.add_array_asserts(schema, query_value),
            oas3::spec::SchemaType::Object => self.add_object_asserts(schema, query_value)?,
        };

        Ok(self)
    }

    fn get_asserts(&self) -> Vec<Assert> {
        self.asserts.to_vec()
    }

    fn add_number_asserts(&mut self, schema: Schema, query_value: &hurl_core::ast::QueryValue) {
        self.asserts.push(assert_query_matches_predicate(
            &query_value,
            hurl_core::ast::PredicateFuncValue::IsFloat,
        ));

        match schema.minimum {
            Some(n) => {
                if schema.exclusive_minimum == Some(true) {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::GreaterThan {
                            space0: single_space(),
                            value: serde_num_to_hurl_num(n),
                            operator: true,
                        },
                    ))
                } else {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::GreaterThanOrEqual {
                            space0: single_space(),
                            value: serde_num_to_hurl_num(n),
                            operator: true,
                        },
                    ))
                }
            }
            None => (),
        };

        match schema.maximum {
            Some(n) => {
                if schema.exclusive_maximum == Some(true) {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::LessThan {
                            space0: single_space(),
                            value: serde_num_to_hurl_num(n),
                            operator: true,
                        },
                    ))
                } else {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::LessThanOrEqual {
                            space0: single_space(),
                            value: serde_num_to_hurl_num(n),
                            operator: true,
                        },
                    ))
                }
            }
            None => (),
        };
    }

    fn build_schema_from_allof(&self, schema: Schema) -> Result<Schema, RefError> {
        let mut new_schema = schema.clone();
        let mut props = BTreeMap::new();

        for schema in new_schema.all_of {
            for prop in schema.resolve(self.spec)?.properties {
                props.insert(prop.0, prop.1);
            }
        }

        for prop in new_schema.properties {
            props.insert(prop.0, prop.1);
        }

        new_schema.all_of = vec![];

        new_schema.properties = props;

        Ok(new_schema)
    }

    fn add_int_asserts(&mut self, schema: Schema, query_value: &hurl_core::ast::QueryValue) {
        self.asserts.push(assert_query_matches_predicate(
            &query_value,
            hurl_core::ast::PredicateFuncValue::IsInteger,
        ));

        match schema.minimum {
            Some(n) => {
                if schema.exclusive_minimum == Some(true) {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::GreaterThan {
                            space0: single_space(),
                            value: predicate_integer_number(n),
                            operator: true,
                        },
                    ))
                } else {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::GreaterThanOrEqual {
                            space0: single_space(),
                            value: predicate_integer_number(n),
                            operator: true,
                        },
                    ))
                }
            }
            None => (),
        };

        match schema.maximum {
            Some(n) => {
                if schema.exclusive_maximum == Some(true) {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::LessThan {
                            space0: single_space(),
                            value: predicate_integer_number(n),
                            operator: true,
                        },
                    ))
                } else {
                    self.asserts.push(assert_query_matches_predicate(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::LessThanOrEqual {
                            space0: single_space(),
                            value: predicate_integer_number(n),
                            operator: true,
                        },
                    ))
                }
            }
            None => (),
        };
    }

    fn add_string_asserts(&mut self, schema: Schema, query_value: &hurl_core::ast::QueryValue) {
        for assert in parse_string_asserts(schema, query_value) {
            self.asserts.push(assert)
        }
    }

    fn add_array_asserts(&mut self, schema: Schema, query_value: &hurl_core::ast::QueryValue) {
        self.asserts.push(assert_query_matches_predicate(
            &query_value,
            hurl_core::ast::PredicateFuncValue::IsCollection,
        ));

        match schema.min_items {
            Some(n) => match n.try_into() {
                Ok(num) => self
                    .asserts
                    .push(assert_query_matches_predicate_with_filters(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::GreaterThanOrEqual {
                            space0: single_space(),
                            value: PredicateValue::Number(hurl_core::ast::Number::Integer(num)),
                            operator: true,
                        },
                        vec![hurl_core::ast::Filter {
                            source_info: empty_source_info(),
                            value: FilterValue::Count,
                        }],
                        None,
                    )),
                Err(e) => {
                    warn!(
                        "minItems for {} can't be used in hurl it is likely too large {}",
                        schema.title.clone().unwrap_or("schema".to_string()),
                        e
                    )
                }
            },
            None => (),
        };

        match schema.max_items {
            Some(n) => match n.try_into() {
                Ok(num) => self
                    .asserts
                    .push(assert_query_matches_predicate_with_filters(
                        &query_value,
                        hurl_core::ast::PredicateFuncValue::LessThanOrEqual {
                            space0: single_space(),
                            value: PredicateValue::Number(hurl_core::ast::Number::Integer(num)),
                            operator: true,
                        },
                        vec![hurl_core::ast::Filter {
                            source_info: empty_source_info(),
                            value: FilterValue::Count,
                        }],
                        None,
                    )),
                Err(e) => {
                    warn!(
                        "maxItems for {} can't be used in hurl it is likely too large {}",
                        schema.title.unwrap_or("schema".to_string()),
                        e
                    )
                }
            },
            None => (),
        };
    }

    fn add_object_asserts(
        &mut self,
        schema: Schema,
        query_value: &hurl_core::ast::QueryValue,
    ) -> Result<(), RefError> {
        self.asserts.push(assert_query_matches_predicate(
            query_value,
            hurl_core::ast::PredicateFuncValue::IsCollection,
        ));

        let path = match query_value {
            hurl_core::ast::QueryValue::Jsonpath { space0: _, expr } => {
                format!(
                    "{}",
                    expr.elements
                        .iter()
                        .map(|e| match e {
                            TemplateElement::String { value: _, encoded } => encoded.to_string(),
                            TemplateElement::Expression(_) => "".to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join(""),
                )
            }
            _ => return Ok(()),
        };

        for property in schema.properties {
            match self.handle_unions_by {
                HandleUnionsBy::IgnoringThem => {
                    if schema.required.contains(&property.0) {
                        let _ = self.add_asserts_from_schema(
                            property.1.resolve(self.spec)?,
                            &hurl_core::ast::QueryValue::Jsonpath {
                                space0: single_space(),
                                expr: simple_template(format!("{path}.{}", property.0)),
                            },
                        );
                    } else {
                        debug!("Not generating asserts for property at {}. The property is not required to generate asserts for optional properties use the option `--validation-response full-with-optionals`", format!("{path}.{}", property.0));
                    }
                }
                HandleUnionsBy::TreatingOptionalsAsRequired => {
                    let _ = self.add_asserts_from_schema(
                        property.1.resolve(self.spec)?,
                        &hurl_core::ast::QueryValue::Jsonpath {
                            space0: single_space(),
                            expr: simple_template(format!("{path}.{}", property.0)),
                        },
                    );
                }
            }
        }

        Ok(())
    }
}

fn serde_num_to_hurl_num(n: serde_json::Number) -> hurl_core::ast::PredicateValue {
    hurl_core::ast::PredicateValue::Number(hurl_core::ast::Number::Float(Float {
        value: n.to_string().parse::<f64>().unwrap_or(0.0),
        encoded: n.to_string(),
    }))
}

fn simple_template(element: String) -> hurl_core::ast::Template {
    hurl_core::ast::Template {
        delimiter: Some('"'),
        source_info: empty_source_info(),
        elements: vec![TemplateElement::String {
            value: "".to_string(),
            encoded: element,
        }],
    }
}

fn predicate_integer_number(n: serde_json::Number) -> PredicateValue {
    match n.to_string().parse::<i64>() {
        Ok(num) => hurl_core::ast::PredicateValue::Number(hurl_core::ast::Number::Integer(num)),
        // Fallback to float if not int
        Err(_) => serde_num_to_hurl_num(n),
    }
}

pub fn parse_json_response_body_asserts(
    schema: Schema,
    spec: &Spec,
    handle_unions_by: HandleUnionsBy,
) -> Result<Vec<Assert>, RefError> {
    Ok(SchemaToJsonAssertBuilder::new(
        &mut vec![assert_status_less_than(400)],
        spec,
        &handle_unions_by,
    )
    .add_asserts_from_schema(
        schema,
        &hurl_core::ast::QueryValue::Jsonpath {
            space0: single_space(),
            expr: simple_template("$".to_string()),
        },
    )?
    .get_asserts())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hurl_core::ast::{Assert, Float};
    use oas3::{Schema, Spec};
    use pretty_assertions::assert_eq;
    use serde_json::Number;

    use crate::{
        hurl_files::single_space,
        response::{
            common_asserts::{assert_query_matches_predicate, assert_status_less_than},
            json_asserts::simple_template,
            response_validation::HandleUnionsBy,
        },
    };

    use super::parse_json_response_body_asserts;

    #[test]
    fn parse_json_response_body_with_no_schema_type_returns_empty_asserts() {
        let mut schema = Schema::default();
        schema.schema_type = None;
        let result = parse_json_response_body_asserts(
            schema,
            &Spec::default(),
            HandleUnionsBy::IgnoringThem,
        );
        let expected: Vec<Assert> = vec![assert_status_less_than(400)];

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn parse_json_response_body_with_bool_schema_type_returns_bool_asserts() {
        let mut schema = Schema::default();
        schema.schema_type = Some(oas3::spec::SchemaType::Boolean);
        let result = parse_json_response_body_asserts(
            schema,
            &Spec::default(),
            HandleUnionsBy::IgnoringThem,
        );

        let expected: Vec<Assert> = vec![
            assert_status_less_than(400),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::IsBoolean,
            ),
        ];

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn parse_json_response_body_with_number_returns_number_asserts() {
        let mut schema = Schema::default();

        schema.schema_type = Some(oas3::spec::SchemaType::Number);
        schema.maximum = Some(Number::from_f64(3.0).unwrap());
        schema.exclusive_maximum = Some(true);

        schema.minimum = Some(Number::from_f64(1.0).unwrap());
        schema.exclusive_minimum = Some(false);

        let result = parse_json_response_body_asserts(
            schema,
            &Spec::default(),
            HandleUnionsBy::IgnoringThem,
        );

        let expected: Vec<Assert> = vec![
            assert_status_less_than(400),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::IsFloat,
            ),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::GreaterThanOrEqual {
                    space0: single_space(),
                    value: hurl_core::ast::PredicateValue::Number(hurl_core::ast::Number::Float(
                        Float {
                            value: 1.0,
                            encoded: "1.0".to_string(),
                        },
                    )),
                    operator: true,
                },
            ),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::LessThan {
                    space0: single_space(),
                    value: hurl_core::ast::PredicateValue::Number(hurl_core::ast::Number::Float(
                        Float {
                            value: 3.0,
                            encoded: "3.0".to_string(),
                        },
                    )),
                    operator: true,
                },
            ),
        ];

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn parse_json_response_body_with_int_returns_int_asserts() {
        let mut schema = Schema::default();

        schema.schema_type = Some(oas3::spec::SchemaType::Integer);
        schema.maximum = Some(Number::from_str("3").unwrap());
        schema.exclusive_maximum = Some(true);

        schema.minimum = Some(Number::from_str("1").unwrap());
        schema.exclusive_minimum = Some(false);

        let result = parse_json_response_body_asserts(
            schema,
            &Spec::default(),
            HandleUnionsBy::IgnoringThem,
        );

        let expected: Vec<Assert> = vec![
            assert_status_less_than(400),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::IsInteger,
            ),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::GreaterThanOrEqual {
                    space0: single_space(),
                    value: hurl_core::ast::PredicateValue::Number(hurl_core::ast::Number::Integer(
                        1,
                    )),
                    operator: true,
                },
            ),
            assert_query_matches_predicate(
                &hurl_core::ast::QueryValue::Jsonpath {
                    space0: single_space(),
                    expr: simple_template("$".to_string()),
                },
                hurl_core::ast::PredicateFuncValue::LessThan {
                    space0: single_space(),
                    value: hurl_core::ast::PredicateValue::Number(hurl_core::ast::Number::Integer(
                        3,
                    )),
                    operator: true,
                },
            ),
        ];

        assert_eq!(Ok(expected), result);
    }
}
