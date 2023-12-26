use hurl_core::ast::{
    Entry, HurlFile, LineTerminator, Method, Pos, Request, Response, SourceInfo, Status, Template,
    TemplateElement, Version, VersionValue, Whitespace,
};
use oas3::{
    spec::{
        Components, FromRef, ObjectOrReference, Operation, Parameter, PathItem, RefError,
        SchemaType,
    },
    Schema, Spec,
};

type OApiPath<'a> = (&'a String, &'a PathItem);

pub struct HurlFiles {
    pub hurl_files: Vec<HurlFile>,
    pub errors: Vec<RefError>,
}

pub fn to_hurl_files(path: OApiPath, spec: &Spec, _components: &Option<Components>) -> HurlFiles {
    let mut hurl_files = vec![];
    let mut errors = vec![];

    match &path.1.get {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::GET) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }

    match &path.1.post {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::POST) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }

    match &path.1.put {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::PUT) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }

    match &path.1.patch {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::PATCH) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }

    match &path.1.options {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::OPTIONS) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }


    match &path.1.delete {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::DELETE) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }

    match &path.1.head {
        Some(o) => match to_file(path, &spec, &o, HttpMethod::HEAD) {
            Ok(file) => hurl_files.push(file),
            Err(e) => errors.extend(e),
        },
        None => (),
    }


    return HurlFiles { hurl_files, errors };
}

fn to_file(
    path: OApiPath,
    spec: &Spec,
    operation: &Operation,
    method: HttpMethod,
) -> Result<HurlFile, Vec<RefError>> {
    let param_result_iter = operation.parameters.iter().map(|p| match p {
        ObjectOrReference::Object(p) => Ok(p.clone()),
        ObjectOrReference::Ref { ref_path } => Parameter::from_ref(&spec, &ref_path),
    });

    let errors = param_result_iter
        .clone()
        .filter_map(|p| match p {
            Ok(_) => None,
            Err(e) => Some(e),
        })
        .collect::<Vec<RefError>>();

    if errors.len() > 0 {
        return Err(errors);
    }

    let param_iter = param_result_iter.clone().filter_map(|p| match p {
        Ok(p) => Some(p),
        Err(_) => None,
    });

    let path_params = param_iter.clone().filter(|p| p.location == "path");
    let mut query_params = param_iter.filter(|p| p.location == "query");

    let uri = path_params.fold(path.0.clone(), |uri, param| {
        let schema = &param.schema.unwrap_or(Schema::default());
        uri.replace(
            &("{".to_string() + &param.name + "}"),
            path_param_from_schema_type(schema.schema_type.unwrap_or(SchemaType::String)),
        )
    });

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

    let uri_with_query_params = query_params.fold(uri_with_first_query_param, |uri, param| {
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
                    encoded: format!("{{{{host}}}}{uri_with_query_params}"),
                }],
                source_info: empty_source_info(),
            },
            line_terminator0: LineTerminator {
                space0: Whitespace {
                    value: " ".to_string(),
                    source_info: empty_source_info(),
                },
                comment: None,
                newline: Whitespace {
                    value: " ".to_string(),
                    source_info: empty_source_info(),
                },
            },
            headers: vec![],
            sections: vec![],
            body: None,
            source_info: empty_source_info(),
        },
        response: Some(Response {
            line_terminators: vec![newline(), newline()],
            version: Version {
                value: VersionValue::VersionAny,
                source_info: empty_source_info(),
            },
            space0: Whitespace {
                value: "".to_string(),
                source_info: empty_source_info(),
            },
            status: Status {
                value: hurl_core::ast::StatusValue::Specific(200),
                source_info: empty_source_info(),
            },
            space1: Whitespace {
                value: " ".to_string(),
                source_info: empty_source_info(),
            },
            line_terminator0: newline(),
            headers: vec![],
            sections: vec![],
            body: None,
            source_info: empty_source_info(),
        }),
    };

    Ok(HurlFile {
        entries: vec![entry],
        line_terminators: vec![],
    })
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

fn empty_source_info() -> SourceInfo {
    SourceInfo {
        start: Pos { column: 0, line: 0 },
        end: Pos { column: 0, line: 0 },
    }
}

fn newline() -> LineTerminator {
    LineTerminator {
        space0: Whitespace {
            value: "".to_string(),
            source_info: empty_source_info(),
        },

        comment: None,
        newline: Whitespace {
            value: "\n".to_string(),
            source_info: SourceInfo {
                start: Pos { column: 0, line: 0 },
                end: Pos { column: 0, line: 0 },
            },
        },
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
    fn to_string(self) -> String {
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
