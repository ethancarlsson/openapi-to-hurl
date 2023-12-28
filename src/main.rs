use std::{fs::File, io::Write};

use crate::{cli::Cli, oai_path::to_hurl_files};
use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::Arguments;

mod cli;
mod oai_path;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let args = cli.args();
    let hurl_files = hurl_files_from_spec_path(&args)?;
    for file_contents in hurl_files {
        let file_path = format!("{}/{}.hurl", args.out.display(), file_contents.0);
        let mut file = File::create(&file_path)
            .with_context(|| format!("Could not open new file at {file_path}"))?;
        file.write_all(file_contents.1.as_bytes())
            .with_context(|| format!("could not write to file at {file_path}"))?;
    }

    Ok(())
}

fn hurl_files_from_spec_path(args: &Arguments) -> Result<Vec<(String, String)>, anyhow::Error> {
    let spec =
        oas3::from_path(args.path.clone()).with_context(|| format!("Issue with specification"))?;

    let mut files = vec![];
    for path in spec.paths.iter() {
        let hurl_files = to_hurl_files(path, &spec, &args);

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

        for file in hurl_files.hurl_files {
            files.push((
                path.0.replace("/", "_"),
                (hurlfmt::format::format_text(file, false)),
            ));
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{cli::Arguments, hurl_files_from_spec_path};

    #[test]
    fn hurl_files_from_spec_path_with_pet_store_spec() {
        let result = hurl_files_from_spec_path(&Arguments {
            path: PathBuf::from_str("test_files/pet_store.json").unwrap(),
            out: PathBuf::from_str("test").unwrap(),
            validate_response: crate::cli::ResponseValidationChoice::Http200,
        });

        let expected: Vec<(String, String)> = vec![
            (
                "_pets".to_string(),
                "GET {{host}}/pets?limit=3\n\nHTTP 200\n".to_string(),
            ),
            (
                "_pets".to_string(),
                "POST {{host}}/pets\n\nHTTP 200\n".to_string(),
            ),
            (
                "_pets_{petId}".to_string(),
                "GET {{host}}/pets/string_value\n\nHTTP 200\n".to_string(),
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn hurl_files_from_spec_no_response_validation() {
        let result = hurl_files_from_spec_path(&Arguments {
            path: PathBuf::from_str("test_files/pet_store.json").unwrap(),
            out: PathBuf::from_str("test").unwrap(),
            validate_response: crate::cli::ResponseValidationChoice::No,
        });

        let expected: Vec<(String, String)> = vec![
            ("_pets".to_string(), "GET {{host}}/pets?limit=3".to_string()),
            ("_pets".to_string(), "POST {{host}}/pets".to_string()),
            (
                "_pets_{petId}".to_string(),
                "GET {{host}}/pets/string_value".to_string(),
            ),
        ];
        assert_eq!(expected, result.unwrap());
    }
}
