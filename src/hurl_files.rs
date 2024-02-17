use crate::{
    custom_hurl_ast::{empty_source_info, empty_space, newline},
    request_body::request_body::SpecBodySettings,
    response::response_validation::{validate_response_not_error, validation_response_full}
};
use hurl_core::ast::{
    Body, EncodedString, Entry, HurlFile, KeyValue, Method, Request, Response, Status, Template,
    TemplateElement, Version, VersionValue, Whitespace,
};
use log::{trace, warn};
use oas3::{
    spec::{
        FromRef, ObjectOrReference, Operation, Parameter, PathItem, RefError, RequestBody,
        SchemaType,
    },
    Schema, Spec,
};

use crate::cli::Settings;
use crate::request_body;

type OApiPath<'a> = (&'a String, &'a PathItem);

pub struct HurlFiles {
    pub hurl_files: Vec<LocalHurlFile>,
    pub errors: Vec<RefError>,
}

pub struct LocalHurlFile {
    pub operation: Option<String>,
    pub method: String,
    pub file: HurlFile,
}

impl HurlFiles {
    pub fn from_oai_path(path: OApiPath, spec: &Spec, args: &Settings) -> HurlFiles {
        HurlFileBuilder::new(&path, spec, args)
            .add_operation(&path.1.get, &HttpMethod::GET)
            .add_operation(&path.1.post, &HttpMethod::POST)
            .add_operation(&path.1.put, &HttpMethod::PUT)
            .add_operation(&path.1.patch, &HttpMethod::PATCH)
            .add_operation(&path.1.options, &HttpMethod::OPTIONS)
            .add_operation(&path.1.delete, &HttpMethod::DELETE)
            .add_operation(&path.1.head, &HttpMethod::HEAD)
            .to_hurl_files()
    }
}

struct HurlFileBuilder<'a> {
    hurl_files: Vec<LocalHurlFile>,
    errors: Vec<RefError>,
    path: &'a OApiPath<'a>,
    spec: &'a Spec,
    args: &'a Settings,
}

fn share_element(vec1: &Vec<String>, vec2: &Vec<String>) -> bool {
    for item in vec1 {
        if vec2.contains(item) {
            return true;
        }
    }
    false
}


impl<'a> HurlFileBuilder<'a> {
    pub fn new(path: &'a OApiPath, spec: &'a Spec, args: &'a Settings) -> HurlFileBuilder<'a> {
        Self {
            hurl_files: vec![],
            errors: vec![],
            path,
            spec,
            args,
        }
    }

    pub fn add_operation(mut self, operation: &Option<Operation>, method: &HttpMethod) -> Self {
        let o = match operation {
            Some(o) => o,
            None => return self,
        };

        let is_in_tag = match &self.args.tags {
            Some(ts) => share_element(ts, &o.tags),
            None => true,
        };

        let is_selected_operation_id = match &self.args.operation_id_selection {
            Some(selection) => {
                // TODO: Figure out how to avoid the clone here.
                selection.contains(&o.clone().operation_id.unwrap_or("no_id".to_string()))
            }
            None => true,
        };

        if !(is_in_tag && is_selected_operation_id) {
            return self;
        }

        match to_file(*self.path, self.spec, &o, &method, self.args) {
            Ok(file) => self.hurl_files.push(LocalHurlFile {
                file,
                method: method.to_string(),
                operation: o.operation_id.clone(),
            }),
            Err(e) => self.errors.extend(e),
        }

        self
    }

    pub fn to_hurl_files(self) -> HurlFiles {
        HurlFiles {
            hurl_files: self.hurl_files,
            errors: self.errors,
        }
    }
}

fn to_file(
    path: OApiPath,
    spec: &Spec,
    operation: &Operation,
    method: &HttpMethod,
    settings: &Settings,
) -> Result<HurlFile, Vec<RefError>> {
    let param_result_iter = operation.parameters.iter().map(|p| match p {
        ObjectOrReference::Object(p) => Ok(p.clone()),
        ObjectOrReference::Ref { ref_path } => Parameter::from_ref(&spec, &ref_path),
    });

    let mut errors = param_result_iter
        .clone()
        .filter_map(|p| match p {
            Ok(_) => None,
            Err(e) => Some(e),
        })
        .collect::<Vec<RefError>>();

    let request_body =
        match parse_request_body(operation, spec, SpecBodySettings::from_settings(settings)) {
            Ok(r) => r,
            Err(e) => {
                errors.push(e);
                return Err(errors);
            }
        };

    if errors.len() > 0 {
        return Err(errors);
    }

    let param_iter = param_result_iter.clone().filter_map(|p| match p {
        Ok(p) => Some(p),
        Err(_) => None,
    });

    let path_params = param_iter.clone().filter(|p| p.location == "path");
    let mut query_params = param_iter.filter(|p| p.location == "query");

    let mut uri = path_params.fold(path.0.clone(), |uri, param| {
        let schema = &param.schema.unwrap_or(Schema::default());
        uri.replace(
            &("{".to_string() + &param.name + "}"),
            path_param_from_schema_type(schema.schema_type.unwrap_or(SchemaType::String)),
        )
    });

    match settings.query_params_choice {
        crate::cli::QueryParamChoice::None => (),
        crate::cli::QueryParamChoice::Defaults => {
            let uri_with_first_query_param = format!(
                "{uri}{}",
                match query_params.next() {
                    Some(param) => {
                        let schema = param.schema.unwrap_or(Schema {
                            example: None,
                            ..Schema::default()
                        });
                        format!(
                            "?{}={}",
                            param.name,
                            match schema.example {
                                Some(e) => e.to_string().replace("\"", ""),
                                None => path_param_from_schema_type(
                                    schema.schema_type.unwrap_or(SchemaType::String)
                                )
                                .to_string(),
                            }
                        )
                    }
                    None => "".to_string(),
                }
            );

            uri = query_params.fold(uri_with_first_query_param, |uri, param| {
                format!(
                    "{uri}&{}={}",
                    param.name,
                    path_param_from_schema_type(
                        param
                            .schema
                            .unwrap_or(Schema::default())
                            .schema_type
                            .unwrap_or(SchemaType::String)
                    )
                )
            });
        }
    };

    let entry = Entry {
        request: Request {
            line_terminators: vec![],
            space0: Whitespace {
                value: "".to_string(),
                source_info: empty_source_info(),
            },
            method: Method(method.to_string()),
            space1: Whitespace {
                value: " ".to_string(),
                source_info: empty_source_info(),
            },
            url: Template {
                delimiter: None,
                elements: vec![TemplateElement::String {
                    value: "".to_string(),
                    encoded: format!("{{{{host}}}}{uri}"),
                }],
                source_info: empty_source_info(),
            },
            line_terminator0: newline(),
            headers: settings
                .custom_variables
                .headers
                .iter()
                .map(|kv| KeyValue {
                    key: EncodedString {
                        value: kv.0.clone(),
                        encoded: kv.0.clone(),
                        quotes: false,
                        source_info: empty_source_info(),
                    },
                    value: Template {
                        delimiter: None,
                        elements: vec![TemplateElement::String {
                            value: "".to_string(),
                            encoded: format!("{{{{{}}}}}", kv.0.clone()),
                        }],
                        source_info: empty_source_info(),
                    },
                    line_terminators: vec![],
                    space0: empty_space(),
                    space1: empty_space(),
                    space2: single_space(),
                    line_terminator0: newline(),
                })
                .collect(),
            sections: vec![],
            body: request_body,
            source_info: empty_source_info(),
        },
        response: match settings.validate_response {
            crate::cli::ResponseValidationChoice::No => None,
            crate::cli::ResponseValidationChoice::Http200 => {
                warn!("Using deprecated option `--validate-response http-200`");
                Some(status_code_200_response())
            },
            crate::cli::ResponseValidationChoice::NonError => Some(validate_response_not_error()),
            crate::cli::ResponseValidationChoice::Full => match validation_response_full(operation, spec, &settings.content_type) {
                Ok(response) => response,
                Err(_) => todo!(),
            }
        },
    };

    Ok(HurlFile {
        entries: vec![entry],
        line_terminators: vec![],
    })
}

fn parse_request_body(
    operation: &Operation,
    spec: &Spec,
    settings: SpecBodySettings,
) -> Result<Option<Body>, RefError> {
    let operation_id = operation
        .operation_id
        .clone()
        .unwrap_or("operationWithNoId".to_string());

    let request_body = match &operation.request_body {
        Some(b) => b,
        None => {
            trace!("No request body found for {}", operation_id);
            return Ok(None);
        }
    };

    trace!(
        "Parsing request body for operation {}",
        operation
            .operation_id
            .clone()
            .unwrap_or("operationWithNoId".to_string())
    );

    let body = match request_body {
        ObjectOrReference::Object(b) => Ok(b.clone()),
        ObjectOrReference::Ref { ref_path } => RequestBody::from_ref(&spec, &ref_path),
    }?;

    Ok(request_body::request_body::from_spec_body(
        body,
        spec,
        operation_id,
        settings,
    )?)
}

fn status_code_200_response() -> Response {
    Response {
        line_terminators: vec![newline(), newline()],
        version: Version {
            value: VersionValue::VersionAny,
            source_info: empty_source_info(),
        },
        space0: empty_space(),
        status: Status {
            value: hurl_core::ast::StatusValue::Specific(200),
            source_info: empty_source_info(),
        },
        space1: single_space(),
        line_terminator0: newline(),
        headers: vec![],
        sections: vec![],
        body: None,
        source_info: empty_source_info(),
    }
}

fn path_param_from_schema_type(schema_type: SchemaType) -> &'static str {
    match schema_type {
        SchemaType::Boolean => "true",
        SchemaType::Integer => "3",
        SchemaType::Number => "5.5",
        SchemaType::String => "string_value",
        SchemaType::Array => "[]array_value",
        SchemaType::Object => "{}",
    }
}

pub fn single_space() -> Whitespace {
    Whitespace {
        value: " ".to_string(),
        source_info: empty_source_info(),
    }
}

enum HttpMethod {
    GET,
    PUT,
    POST,
    PATCH,
    OPTIONS,
    HEAD,
    DELETE,
}

impl HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
            HttpMethod::OPTIONS => "OPTIONS".to_string(),
            HttpMethod::HEAD => "HEAD".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
        }
    }
}
