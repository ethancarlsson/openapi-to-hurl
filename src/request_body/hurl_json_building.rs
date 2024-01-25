use hurl_core::ast::{JsonValue, Template};

use crate::cli::Formatting;

pub fn build_json_object_element(
    name: Template,
    value: JsonValue,
    depth: usize,
    formatting: &Formatting,
) -> hurl_core::ast::JsonObjectElement {
    match formatting {
        Formatting::NoFormatting => hurl_core::ast::JsonObjectElement {
            space0: "".to_string(),
            name,
            space1: "".to_string(),
            space2: "".to_string(),
            value,
            space3: "".to_string(),
        },
        Formatting::RequestBodies => hurl_core::ast::JsonObjectElement {
            space0: "\n".to_string() + &"  ".repeat(depth),
            name,
            space1: "".to_string(),
            space2: " ".to_string(),
            value,
            space3: "".to_string(),
        },
    }
}

pub fn build_json_list_space(
    formatting: &Formatting,
) -> String {
    match  formatting {
        Formatting::NoFormatting => "".to_string(),
        Formatting::RequestBodies => " ".to_string(),
    }
}

pub fn build_json_list_value(
    value: JsonValue,
    formatting: &Formatting,
) -> hurl_core::ast::JsonListElement {
    match formatting {
        Formatting::NoFormatting => hurl_core::ast::JsonListElement {
            space0: "".to_string(),
            value,
            space1: "".to_string(),
        },
        Formatting::RequestBodies => hurl_core::ast::JsonListElement {
            space0: " ".to_string(),
            value,
            space1: " ".to_string(),
        },
    }
}
