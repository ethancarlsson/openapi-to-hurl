use crate::variable_files::VariableFiles;
use std::{
    fs::{self, File},
    io::Write,
};

use crate::cli::Cli;
use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::Arguments;
use hurl_files::HurlFiles;
use oas3::Spec;

mod cli;
mod hurl_files;
mod variable_files;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let args = cli.args();

    let spec =
        oas3::from_path(args.path.clone()).with_context(|| format!("Issue with specification"))?;

    let hurl_files = hurl_files_from_spec_path(&args, &spec)?;

    for file_contents in hurl_files {
        fs::create_dir(format!(
            "{}/{}",
            args.out.display(),
            file_contents.0.clone()
        ))?;

        for file_string in file_contents.1 {
            let file_path = format!(
                "{}/{}/{}.hurl",
                args.out.display(),
                file_contents.0,
                file_string.method
            );
            let mut file = File::create(&file_path)
                .with_context(|| format!("Could not open new file at {file_path}"))?;

            file.write_all(file_string.file.as_bytes())
                .with_context(|| format!("could not write to file at {file_path}"))?;
        }
    }

    for v_file in VariableFiles::from_spec(&spec, args.custom_variables).files {
        let file_path = format!("{}/{}", args.out.display(), v_file.name);
        let mut file = File::create(&file_path)
            .with_context(|| format!("Could not open new file at {file_path}"))?;
        file.write_all(v_file.get_contents().as_bytes())
            .with_context(|| format!("could not write to file at {file_path}"))?;
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct HurlFileString {
    pub method: String,
    pub file: String,
}

fn hurl_files_from_spec_path(
    args: &Arguments,
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

        files.push((
            path.0.replace("/", "_"),
            hurl_files
                .hurl_files
                .iter()
                .map(|f| HurlFileString {
                    method: f.method.clone(),
                    file: hurlfmt::format::format_text(f.file.clone(), false),
                })
                .collect(),
        ))
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{
        cli::Arguments, hurl_files_from_spec_path, variable_files::CustomVariables, HurlFileString,
    };

    #[test]
    fn hurl_files_from_spec_path_with_pet_store_spec() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();

        let result = hurl_files_from_spec_path(
            &Arguments {
                path: spec_path,
                out: PathBuf::from_str("test").unwrap(),
                validate_response: crate::cli::ResponseValidationChoice::Http200,
                custom_variables: CustomVariables { headers: vec![] },
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![
                    HurlFileString {
                        file: "GET {{host}}/pets?limit=3\n\n\nHTTP 200\n".to_string(),
                        method: "GET".to_string(),
                    },
                    HurlFileString {
                        file: "POST {{host}}/pets\n\n\nHTTP 200\n".to_string(),
                        method: "POST".to_string(),
                    },
                ],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/string_value\n\n\nHTTP 200\n".to_string(),
                    method: "GET".to_string(),
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
            &Arguments {
                path: spec_path,
                out: PathBuf::from_str("test").unwrap(),
                validate_response: crate::cli::ResponseValidationChoice::No,
                custom_variables: CustomVariables { headers: vec![] },
            },
            &spec,
        );

        let expected: Vec<(String, Vec<HurlFileString>)> = vec![
            (
                "_pets".to_string(),
                vec![
                    HurlFileString {
                        file: "GET {{host}}/pets?limit=3\n".to_string(),
                        method: "GET".to_string(),
                    },
                    HurlFileString {
                        file: "POST {{host}}/pets\n".to_string(),
                        method: "POST".to_string(),
                    },
                ],
            ),
            (
                "_pets_{petId}".to_string(),
                vec![HurlFileString {
                    file: "GET {{host}}/pets/string_value\n".to_string(),
                    method: "GET".to_string(),
                }],
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }
}
