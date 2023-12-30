use std::error::Error;

use clap::{Parser, ValueEnum};

use crate::variable_files::CustomVariables;

#[derive(ValueEnum, Clone)]
pub enum ResponseValidationChoice {
    /// No Validation
    No,
    /// HTTP Status Code
    Http200,
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
pub struct Cli {
    /// The path to the openapi specification
    path: std::path::PathBuf,
    /// Directory where the hurl files will be created
    out: std::path::PathBuf,
    /// Response validation
    #[arg(short = 'r', long, default_value_t = ResponseValidationChoice::Http200, value_enum)]
    validate_response: ResponseValidationChoice,
    /// Input: `HEADER_KEY=HEADER_VALUE`. Custom headers will be added to each request as `HEADER_KEY: {{HEADER_KEY}}`
    /// and to the variables file as `HEADER_KEY=HEADER_VALUE`
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    header_vars: Vec<(String, String)>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub struct Arguments {
    pub path: std::path::PathBuf,
    pub out: std::path::PathBuf,
    pub validate_response: ResponseValidationChoice,
    pub custom_variables: CustomVariables,
}

impl Cli {
    pub fn args(self) -> Arguments {
        Arguments {
            path: self.path,
            out: self.out,
            validate_response: self.validate_response,
            custom_variables: CustomVariables {
                headers: self.header_vars,
            },
        }
    }
}
