use crate::variable_files::{VariableFile, VariableFiles};
use std::{
    fs::{self, File},
    io::Write,
};

use crate::cli::Cli;
use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::Settings;
use hurl_files::HurlFiles;
use log::trace;
use oas3::Spec;

mod cli;
mod custom_hurl_ast;
mod hurl_files;
mod request_body;
mod variable_files;

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let args = cli.args()?;
    trace!("parsing oas3 from path");
    let spec =
        oas3::from_path(args.path.clone()).with_context(|| format!("Invalid Open API 3.1 Specification. This tool only aims to support OpenAPI 3.1 and up."))?;

    trace!("transforming oas3 to hurl files");
    let hurl_files = hurl_files_from_spec_path(&args, &spec)?;
    trace!("transforming oas3 to hurl variables file");
    let variable_files = VariableFiles::from_spec(&spec, args.custom_variables);

    trace!("returning values out");
    match args.out {
        cli::OutStrategy::Console => out_to_console(hurl_files)?,
        cli::OutStrategy::Files(out_path) => out_to_files(hurl_files, variable_files, out_path)?,
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
) -> Result<()> {
    let mut files_created_count = 0;
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
                .with_context(|| format!("Could not open file at {file_path}. Most likely because the directory `{}` does not exist", out_path.display()))?;

            file.write_all(file_string.file.as_bytes())
                .with_context(|| format!("Could not write to file at {file_path}"))?;
            files_created_count += 1
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

    println!("Created or updated {files_created_count} hurl files");

    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct HurlFileString {
    pub filename: String,
    pub file: String,
}

fn hurl_files_from_spec_path(
    args: &Settings,
    spec: &Spec,
) -> Result<Vec<(String, Vec<HurlFileString>)>, anyhow::Error> {
    let mut files = vec![];
    for path in spec.paths.iter() {
        let hurl_files = HurlFiles::from_oai_path(path, &spec, &args);

        if hurl_files.errors.len() > 0 {
            bail!(
                "Found errors while parsing openapi file:\n\n{}",
                hurl_files
                    .errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }

        if hurl_files.hurl_files.len() > 0 {
            files.push((
                path.0.replace("/", "_"),
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

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{
        cli::{Formatting, QueryParamChoice, ResponseValidationChoice, Settings},
        hurl_files_from_spec_path,
        variable_files::CustomVariables,
        HurlFileString,
    };

    #[test]
    fn hurl_files_from_spec_path_with_pet_store_spec() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Settings {
                path: spec_path,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![
                    HurlFileString {
                        file: "GET {{host}}/pets?limit=3\n\n\nHTTP 200\n".to_string(),
                        filename: "listPets".to_string(),
                    },
                    HurlFileString {
                        file: "POST {{host}}/pets\n{\n  \"id\": 3,\n  \"name\": \"string\",\n  \"photo_urls\": [  \"https://example.com/img.png\" , \"https://example.com/img2.png\" ],\n  \"tag\": \"string\"}\n\n\nHTTP 200\n".to_string(),
                        filename: "addPet".to_string(),
                    },
                ],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/string_value\n\n\nHTTP 200\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
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
                path: spec_path,
                operation_id_selection: Some(vec!["listPets".to_string()]),
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "GET {{host}}/pets?limit=3\n\n\nHTTP 200\n".to_string(),
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
                path: spec_path,
                operation_id_selection: Some(vec!["addPet".to_string()]),
                formatting: Formatting::NoFormatting,
                ..Settings::default()
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![(
            "_pets".to_string(),
            vec![HurlFileString {
                file: "POST {{host}}/pets\n{\"id\":3,\"name\":\"string\",\"photo_urls\":[\"https://example.com/img.png\",\"https://example.com/img2.png\"],\"tag\":\"string\"}\n\n\nHTTP 200\n".to_string(),
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
                path: spec_path,
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
                    file: "GET {{host}}/pets\n\n\nHTTP 200\n".to_string(),
                    filename: "listPets".to_string(),
                }],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/string_value\n\n\nHTTP 200\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_no_response_validation() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();
        let result = hurl_files_from_spec_path(
            &Settings {
                path: spec_path,
                validate_response: crate::cli::ResponseValidationChoice::No,
                query_params_choice: crate::cli::QueryParamChoice::Defaults,
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
                    file: "GET {{host}}/pets?limit=3\n".to_string(),
                    filename: "listPets".to_string(),
                }],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/string_value\n".to_string(),
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
                path: spec_path,
                custom_variables: CustomVariables {
                    headers: vec![
                        ("Authorization".to_string(), "Bearer test".to_string()),
                        ("test_key".to_string(), "test_val".to_string()),
                    ],
                },
                query_params_choice: QueryParamChoice::None,
                validate_response: ResponseValidationChoice::No,
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
                    file: "GET {{host}}/pets/string_value\nAuthorization: {{Authorization}}\ntest_key: {{test_key}}\n".to_string(),
                    filename: "showPetById".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }
}
