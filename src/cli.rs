use std::error::Error;

use anyhow::anyhow;
use clap::{Parser, ValueEnum};

use crate::variable_files::CustomVariables;

#[derive(ValueEnum, Clone, Default)]
pub enum ResponseValidationChoice {
    /// No Validation
    No,
    #[default]
    /// HTTP Status Code
    Http200,
}

#[derive(ValueEnum, Clone, Default)]
pub enum QueryParamChoice {
    /// No query params
    None,
    #[default]
    /// Default values based on types
    Defaults,
}

#[derive(ValueEnum, Clone, Default)]
pub enum VariablesUpdateStrategy {
    /// Overwrites the entire variables file with new variables
    Overwrite,
    #[default]
    /// Merges new variables with old variables
    Merge,
}

#[derive(ValueEnum, Clone, Default)]
pub enum OutputTo {
    /// If console is selected prints all hurl files to console. NOTE: Does not
    /// print variables to console.
    Console,
    /// If file is selected "out" is required. NOTE: A variables file will also
    /// be created.
    #[default]
    Files,
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
pub struct Cli {
    /// Where will the output go
    #[arg(long, default_value_t = OutputTo::default(), value_enum)]
    output_to: OutputTo,
    /// The path to the openapi specification
    path: std::path::PathBuf,
    /// Directory where the hurl files will be created
    out: Option<std::path::PathBuf>,
    /// Response validation
    #[arg(short = 'r', long, default_value_t = ResponseValidationChoice::default(), value_enum)]
    validate_response: ResponseValidationChoice,
    /// Input: `HEADER_KEY=HEADER_VALUE`. Custom headers will be added to each request as `HEADER_KEY: {{HEADER_KEY}}`
    /// and to the variables file as `HEADER_KEY=HEADER_VALUE`
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    header_vars: Vec<(String, String)>,
    /// Lets you choose whether, and how to, pass query params
    #[arg(short = 'q', long, default_value_t = QueryParamChoice::default(), value_enum)]
    query_params: QueryParamChoice,
    /// Select an operationId from Open API Spec, can select multiple operationIds
    #[arg(short = 'i', long)]
    select_operation_id: Option<Vec<String>>,
    /// How the variables file should be updated
    #[arg(long, default_value_t = VariablesUpdateStrategy::default(), value_enum)]
    variables_update_strategy: VariablesUpdateStrategy,
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

#[derive(Default)]
pub enum OutStrategy {
    // Default here for convenience in unit tests, real default is files.
    #[default]
    Console,
    Files(std::path::PathBuf),
}

#[derive(Default)]
pub struct Arguments {
    pub path: std::path::PathBuf,
    pub out: OutStrategy,
    pub validate_response: ResponseValidationChoice,
    pub query_params_choice: QueryParamChoice,
    pub custom_variables: CustomVariables,
    pub variables_update_strategy: VariablesUpdateStrategy,
    pub operation_id_selection: Option<Vec<String>>,
}

impl Cli {
    pub fn args(self) -> Result<Arguments, anyhow::Error> {
        Ok(Arguments {
            path: self.path,
            out: match self.output_to {
                OutputTo::Console => OutStrategy::Console,
                OutputTo::Files => OutStrategy::Files(match self.out {
                    Some(f) => f,
                    None => {
                        return Err(anyhow!(
                            "Option `out` is required if `--output_to files` option is selected"
                        ))
                    }
                }),
            },
            validate_response: self.validate_response,
            query_params_choice: self.query_params,
            variables_update_strategy: self.variables_update_strategy,
            custom_variables: CustomVariables {
                headers: self.header_vars,
            },
            operation_id_selection: self.select_operation_id,
        })
    }
}
