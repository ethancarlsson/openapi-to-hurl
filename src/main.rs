use crate::{
    settings::Settings,
    variable_files::{VariableFile, VariableFiles},
};
use std::{
    fs::{self, File},
    io::{self, IsTerminal, Write},
};

use crate::cli::Cli;
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use cli::{ErrorHandling, Grouping};
use errors::OperationError;
use hurl_files::HurlFiles;
use log::{error, info, trace};
use oas3::Spec;

mod cli;
mod content_type;
mod custom_hurl_ast;
mod errors;
mod hurl_files;
mod request_body;
mod response;
mod settings;
mod spec_reader;
mod variable_files;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let grouping = cli.grouping.clone();

    if cli.version {
        return Ok(println!(env!("CARGO_PKG_VERSION")));
    }

    let args: Settings = cli.try_into()?;

    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.log_level.clone() as usize)
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    trace!("parsing oas3 from path");

    let spec = match &args.input {
        Some(p) => spec_reader::from_path(p.to_path_buf())?,
        None => {
            let stdin = io::stdin().lock();

            if stdin.is_terminal() {
                return Err(anyhow!("Input can be either the path to an Open API specification file or it can be the entire specification passed in to stdin\n\nUsage: openapi-to-hurl <INPUT> [OUTPUT]\n\nFor example `openapi-to-hurl path/to/openapi/spec.json` or `cat path/to/openapi/spec.json | openapi-to-hurl`\n\nFor more information, try '--help'."));
            }

            spec_reader::from_reader(stdin)?
        }
    };

    trace!("transforming oas3 to hurl files");
    let hurl_files = hurl_files_from_spec_path(&args, &spec)?;
    trace!("transforming oas3 to hurl variables file");
    let variable_files = VariableFiles::from_spec(&spec, args.custom_variables);

    trace!("returning values out");
    match args.out_dir {
        Some(out_dir) => out_to_files(hurl_files, variable_files, out_dir, grouping)?,
        None => out_to_console(hurl_files)?,
    };

    Ok(())
}

fn out_to_console(hurl_files: Vec<(String, Vec<HurlFileString>)>) -> Result<()> {
    for file_contents in hurl_files {
        for file_string in file_contents.1 {
            println!("{}", file_string.file);
        }
    }
    Ok(())
}

fn out_to_files(
    hurl_files: Vec<(String, Vec<HurlFileString>)>,
    variable_files: VariableFiles,
    out_path: std::path::PathBuf,
    grouping: Grouping,
) -> Result<()> {
    let mut files_created_count = 0;
    match grouping {
        Grouping::Flat => {
            for file_contents in hurl_files {
                for file_string in file_contents.1 {
                    let file_path = format!("{}/{}.hurl", out_path.display(), file_string.filename);
                    let mut file = File::create(&file_path)
                        .with_context(|| format!("Could create file {file_path}."))?;

                    file.write_all(file_string.file.as_bytes())
                        .with_context(|| format!("Could not write to file {file_path}"))?;
                    files_created_count += 1
                }
            }
        }
        Grouping::Path => {
            for file_contents in hurl_files {
                let dir_path = format!("{}/{}", out_path.display(), file_contents.0.clone());

                fs::create_dir_all(&dir_path)
                    .with_context(|| format!("couldn't create directory: {dir_path}"))?;

                for file_string in file_contents.1 {
                    let file_path = format!(
                        "{}/{}/{}.hurl",
                        out_path.display(),
                        file_contents.0,
                        file_string.filename
                    );
                    let mut file = File::create(&file_path)
                        .with_context(|| format!("Could create file {file_path}."))?;

                    file.write_all(file_string.file.as_bytes())
                        .with_context(|| format!("Could not write to file {file_path}"))?;
                    files_created_count += 1
                }
            }
        }
    }

    for v_file in variable_files.files {
        let file_path = format!("{}/{}", out_path.display(), v_file.name);
        let existing_variable_file = match fs::read_to_string(&file_path) {
            Ok(f) => VariableFile::from_string(v_file.name.clone(), f),
            Err(_) => VariableFile::empty(v_file.name.clone()),
        };

        let mut file = File::create(&file_path)
            .with_context(|| format!("Could not open file at {file_path}. Most likely because the directory `{}` does not exist", out_path.display()))?;

        file.write_all(
            v_file
                .merge(existing_variable_file)
                .get_contents()
                .as_bytes(),
        )
        .with_context(|| format!("could not write to file at {file_path}"))?;
    }

    info!(
        "Created or updated {files_created_count} hurl files in {}",
        out_path.to_str().unwrap_or("directory")
    );

    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct HurlFileString {
    pub filename: String,
    pub file: String,
}

fn handle_errors(
    errors: Vec<OperationError>,
    error_handling: &ErrorHandling,
) -> Result<(), anyhow::Error> {
    match error_handling {
        ErrorHandling::Log => {
            for err in errors {
                error!("{}", err);
            }
            Ok(())
        }
        ErrorHandling::Terminate => {
            bail!(
                "Found errors while parsing openapi file:\n\n{}",
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
    }
}

fn hurl_files_from_spec_path(
    args: &Settings,
    spec: &Spec,
) -> Result<Vec<(String, Vec<HurlFileString>)>, anyhow::Error> {
    let mut files = vec![];
    for path in spec.paths.iter() {
        for p in path {
            let hurl_files = HurlFiles::from_oai_path(p, &spec, &args);

            if hurl_files.errors.len() > 0 {
                handle_errors(hurl_files.errors, &args.error_handling)?
            }

            if hurl_files.hurl_files.len() > 0 {
                files.push((
                    p.0.replace("/", "_"),
                    hurl_files
                        .hurl_files
                        .iter()
                        .map(|f| HurlFileString {
                            filename: f.operation.clone().unwrap_or(f.method.clone()),
                            file: hurlfmt::format::format_text(f.file.clone(), false),
                        })
                        .collect(),
                ))
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use crate::Settings;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::{path::PathBuf, str::FromStr};

    use crate::{
        cli::{Formatting, QueryParamChoice, ResponseValidationChoice},
        content_type::ContentType,
        hurl_files_from_spec_path,
        variable_files::CustomVariables,
        HurlFileString,
    };

    // Also works for updatePet
    fn get_add_pet_request_body() -> serde_json::Value {
        json!({
        "id": 3,
        "name": "string",
        "photo_urls": [
          "https://example.com/img.png",
          "https://example.com/img2.png"
        ],
        "tag": "string",
        "inner": {
            "test": "string"
        }
          })
    }

    #[test]
    fn hurl_files_from_spec_path_with_pet_store_spec() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                ..Settings::default()
            },
            &spec,
        );

        let add_pet_request_body = get_add_pet_request_body();

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![
                    HurlFileString {
                        file: "GET {{host}}/pets?limit=3\n".to_string(),
                        filename: "listPets".to_string(),
                    },
                    HurlFileString {
                        file: "POST {{host}}/pets\n```json\n".to_string()
                            + &serde_json::to_string_pretty(&add_pet_request_body).unwrap()
                            + "\n```\n",
                        filename: "addPet".to_string(),
                    },
                    HurlFileString {
                        file: "PATCH {{host}}/pets\n```json\n".to_string()
                            + &serde_json::to_string_pretty(&add_pet_request_body).unwrap()
                            + &"\n```\n".to_string(),
                        filename: "updatePet".to_string(),
                    },
                ],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![
                    HurlFileString {
                        file: "GET {{host}}/pets/22\n".to_string(),
                        filename: "showPetById".to_string(),
                    },
                    HurlFileString {
                        file: "POST {{host}}/pets/id_11\n".to_string(),
                        filename: "createPetById".to_string(),
                    },
                ],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_pet_store_spec_and_operation_id_selected() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                operation_id_selection: Some(vec!["listPets".to_string()]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "GET {{host}}/pets?limit=3\n".to_string(),
                filename: "listPets".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_nonerror_validation_selected() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                operation_id_selection: Some(vec!["listPets".to_string()]),
                validate_response: ResponseValidationChoice::NonErrorCode,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "GET {{host}}/pets?limit=3\n\nHTTP *\n[Asserts]\nstatus < 400".to_string(),
                filename: "listPets".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_no_formatting() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                operation_id_selection: Some(vec!["addPet".to_string()]),
                formatting: Formatting::NoFormatting,
                ..Settings::default()
            },
            &spec,
        );

        let add_pet_request_body = get_add_pet_request_body();

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets\n```json\n".to_string()
                    + &add_pet_request_body.to_string()
                    + &"\n```\n".to_string(),
                filename: "addPet".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_pet_store_spec_no_query_params() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec![
                    "listPets".to_string(),
                    "showPetById".to_string(),
                ]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets\n".to_string(),
                    filename: "listPets".to_string(),
                }],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/22\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_plain_text() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["addPet".to_string()]),
                content_type: ContentType::Text,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets\n```\n10,\\\"doggie\\\"\n```\n".to_string(),
                filename: "addPet".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_plain_text_and_full_validation() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["addPet".to_string()]),
                content_type: ContentType::Text,
                validate_response: ResponseValidationChoice::Body,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets\n```\n10,\\\"doggie\\\"\n```\n\nHTTP *\n[Asserts]\n\nstatus < 400\nbody isString\nbody matches /^\\d+,\\d+$/\nbody matches /^.{4}/ #assert min length\nbody matches /^.{0,100}$/ #assert max length".to_string(),
                filename: "addPet".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_json_and_full_validation() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["addPet".to_string()]),
                content_type: ContentType::Json,
                validate_response: ResponseValidationChoice::Body,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets\n```json\n".to_string()
                    + &serde_json::to_string_pretty(&get_add_pet_request_body()).unwrap()
                    + "\n```\n\nHTTP *"
                    + "\n[Asserts]"
                    + "\n\nstatus < 400"
                    + "\njsonpath \"$\" isCollection"
                    + "\njsonpath \"$.id\" isInteger"
                    + "\njsonpath \"$.inner\" isCollection\njsonpath \"$.inner.test\" isString"
                    + "\njsonpath \"$.name\" isString\njsonpath \"$.photo_urls\" isCollection",
                filename: "addPet".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_json_and_full_with_optional_validation() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["addPet".to_string()]),
                content_type: ContentType::Json,
                validate_response: ResponseValidationChoice::BodyWithOptionals,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets\n```json\n".to_string()
                    + &serde_json::to_string_pretty(&get_add_pet_request_body()).unwrap()
                    + "\n```\n\nHTTP *"
                    + "\n[Asserts]"
                    + "\n\nstatus < 400"
                    + "\njsonpath \"$\" isCollection"
                    + "\njsonpath \"$.id\" isInteger"
                    + "\njsonpath \"$.inner\" isCollection\njsonpath \"$.inner.test\" isString"
                    + "\njsonpath \"$.name\" isString\njsonpath \"$.photo_urls\" isCollection"
                    + "\njsonpath \"$.tag\" isString",
                filename: "addPet".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_path_with_plain_text_option_but_no_plain_text_in_schema_selects_first_valid(
    ) {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["updatePet".to_string()]),
                content_type: ContentType::Text,
                formatting: Formatting::NoFormatting,
                ..Settings::default()
            },
            &spec,
        );

        let request_body = get_add_pet_request_body();

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "PATCH {{host}}/pets\n```json\n".to_string()
                    + &request_body.to_string()
                    + &"\n```\n".to_string(),
                filename: "updatePet".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_with_no_response_validation_with_all_query_params() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();
        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                validate_response: crate::cli::ResponseValidationChoice::None,
                query_params_choice: crate::cli::QueryParamChoice::All,
                operation_id_selection: Some(vec![
                    "listPets".to_string(),
                    "showPetById".to_string(),
                ]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets?limit=3&offset=1\n".to_string(),
                    filename: "listPets".to_string(),
                }],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/22\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_with_header_variables() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();
        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                custom_variables: CustomVariables {
                    headers: vec![
                        ("Authorization".to_string(), "Bearer test".to_string()),
                        ("test_key".to_string(), "test_val".to_string()),
                    ],
                },
                query_params_choice: QueryParamChoice::None,
                validate_response: ResponseValidationChoice::None,
                operation_id_selection: Some(vec![
                    "listPets".to_string(),
                    "showPetById".to_string(),
                ]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![
                    HurlFileString {
                        file: "GET {{host}}/pets\nAuthorization: {{Authorization}}\ntest_key: {{test_key}}\n".to_string(),
                        filename: "listPets".to_string(),
                    },
                ],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/22\nAuthorization: {{Authorization}}\ntest_key: {{test_key}}\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_using_tag_filter_returns_only_those_in_tag() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                tags: Some(vec!["petsRead".to_string()]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets\n".to_string(),
                    filename: "listPets".to_string(),
                }],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/22\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_with_no_expected_status_code() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["createPetById".to_string()]),
                validate_response: ResponseValidationChoice::Body,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets_{petId}".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets/id_11\n\nHTTP *\n[Asserts]\n\njsonpath \"$\" isCollection\njsonpath \"$.code\" isInteger\njsonpath \"$.message\" isString".to_string(),
                filename: "createPetById".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_with_no_expected_status_code_plain_text() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["createPetById".to_string()]),
                validate_response: ResponseValidationChoice::Body,
                content_type: ContentType::Text,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets_{petId}".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets/id_11\n\nHTTP *\n[Asserts]\n\nbody isString".to_string(),
                filename: "createPetById".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_using_tag_and_oid_filter_returns_only_those_in_tag_and_operation_id() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec!["showPetById".to_string()]),
                tags: Some(vec!["petsRead".to_string()]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets_{petId}".to_string(),
            vec![HurlFileString {
                file: "GET {{host}}/pets/22\n".to_string(),
                filename: "showPetById".to_string(),
            }],
        )];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_with_path_param_variables_option() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                input: Some(spec_path),
                query_params_choice: crate::cli::QueryParamChoice::None,
                operation_id_selection: Some(vec![
                    "showPetById".to_string(),
                    "createPetById".to_string(),
                ]),
                path_params_choice: crate::cli::PathParamChoice::Variables,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets_{petId}".to_string(),
            vec![
                HurlFileString {
                    file: "GET {{host}}/pets/{{petId}}\n[Options]\nvariable: petId=22\n"
                        .to_string(),
                    filename: "showPetById".to_string(),
                },
                HurlFileString {
                    file: "POST {{host}}/pets/{{petId}}\n[Options]\nvariable: petId=id_11\n"
                        .to_string(),
                    filename: "createPetById".to_string(),
                },
            ],
        )];

        assert_eq!(expected, result.unwrap());
    }
}
