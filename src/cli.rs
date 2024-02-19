use std::error::Error;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Default)]
pub enum ResponseValidationChoice {
    /// No Validation
    No,
    #[default]
    /// HTTP Status Code. WARNING this will be deprecated in version 1
    Http200,
    /// Validates the result is any status code less than 400
    NonError,
    /// Validates the structure and types of the entire response
    /// NOTE: This tool will not produce response validation for union types (nullable, oneOf, not
    /// required, etc).
    Full,
    /// Validates the full strucure and treats all properties in the response body
    /// as if they are required.
    FullWithOptionals,
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

#[derive(ValueEnum, Clone, Default)]
pub enum Formatting {
    /// Will not add any characters to the output that do not affect syntax
    NoFormatting,
    /// Will add some formatting to the request body
    #[default]
    RequestBodies,
}

#[derive(ValueEnum, Clone, Default)]
pub enum LogLevel {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 0,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    #[default]
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}


#[derive(ValueEnum, Clone, Default)]
pub enum CliContentType {
    Text,
    #[default]
    Json,
}

#[derive(ValueEnum, Clone, Default)]
pub enum ErrorHandling {
    /// Log the error to stderr but continue processing. Note that the program will
    /// still terminate if the error is unrecoverable, e.g. the input isn't a valid
    /// Open API Specification.
    Log,
    /// Terminate the program on any errors found with the specification.
    #[default]
    Terminate,
}

/// Generate hurl files from an Open API 3 specification
#[derive(Parser)]
pub struct Cli {
    /// Where will the output go
    #[arg(long, default_value_t = OutputTo::default(), value_enum)]
    pub output_to: OutputTo,
    /// The path to the openapi specification
    pub path: std::path::PathBuf,
    /// Directory where the hurl files will be created
    pub out: Option<std::path::PathBuf>,
    /// Response validation
    #[arg(short = 'r', long, default_value_t = ResponseValidationChoice::default(), value_enum)]
    pub validate_response: ResponseValidationChoice,
    /// Input: `HEADER_KEY=HEADER_VALUE`. Custom headers will be added to each request as `HEADER_KEY: {{HEADER_KEY}}`
    /// and to the variables file as `HEADER_KEY=HEADER_VALUE`
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    pub header_vars: Vec<(String, String)>,
    /// Lets you choose whether, and how to, pass query params
    #[arg(short = 'q', long, default_value_t = QueryParamChoice::default(), value_enum)]
    pub query_params: QueryParamChoice,
    /// Select an operationId from Open API Spec, can select multiple operationIds
    #[arg(short = 'i', long)]
    pub select_operation_id: Option<Vec<String>>,
    /// Filter by tags in the Open API Spec, can select multiple tags. If used with the
    /// "select-operation-id" option the request will first be narrowed by tag then by operationId
    #[arg(short = 't', long)]
    pub tag: Option<Vec<String>>,
    /// How the variables file should be updated
    #[arg(long, default_value_t = VariablesUpdateStrategy::default(), value_enum)]
    pub variables_update_strategy: VariablesUpdateStrategy,
    /// How the output should be formatted
    #[arg(long, default_value_t = Formatting::default(), value_enum)]
    pub formatting: Formatting,
    /// Content type of the request. If the selected content type is not available in the schema or not
    /// supported by this tool the tool will select the first scpecified content type supported by this
    /// tool. If no valid content type is found the tool will use an empty request body.
    #[arg(long, default_value_t = CliContentType::default(), value_enum)]
    pub content_type: CliContentType,
    #[arg(short = 'l', long, default_value_t = LogLevel::default(), value_enum)]
    pub log_level: LogLevel,
    /// Set this to true to silence all logging
    #[arg(long, default_value_t = false)]
    pub quiet: bool,
    /// Set to `log` to log errors and keep generating hurl files where possible.
    #[arg(long, default_value_t = ErrorHandling::default(), value_enum)]
    pub handle_errors: ErrorHandling,
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

