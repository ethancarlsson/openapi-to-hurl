use std::error::Error;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Default)]
pub enum ResponseValidationChoice {
    /// No Validation.
    #[default]
    None,
    /// Validates the result is any status code less than 400.
    NonErrorCode,
    /// Validates the structure and types of the response body.
    /// Note: This tool will not produce response validation for union types (nullable, oneOf, not
    /// required, etc).
    Body,
    /// Validates the response body and treats all properties in the response body
    /// as if they are required.
    BodyWithOptionals,
}

#[derive(ValueEnum, Clone, Default)]
pub enum QueryParamChoice {
    /// No query params.
    None,
    /// Default values based on types.
    #[default]
    Defaults,
}

#[derive(ValueEnum, Clone, Default)]
pub enum VariablesUpdateStrategy {
    /// Overwrites the entire variables file with new variables.
    Overwrite,
    #[default]
    /// Merges new variables with old variables.
    Merge,
}

#[derive(ValueEnum, Clone, Default)]
pub enum Formatting {
    /// Will not add any characters to the output that do not affect syntax.
    NoFormatting,
    /// Will add some formatting to the request body.
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

/// Generate hurl files from an Open API 3 specification.
#[derive(Parser)]
pub struct Cli {
    /// Input can be either a path to the specification or the result of stdin if used in a
    /// pipeline.
    pub input: Option<std::path::PathBuf>,
    /// If the `out-dir` argument is provided the output will go to a series of directories and
    /// files instead of stdout.
    /// Note: if now `out-dir` is given, no variables file will be created.
    #[arg(short = 'o', long)]
    pub out_dir: Option<std::path::PathBuf>,
    /// This option indicates how the response should be validated.
    #[arg(short = 'n', long, default_value_t = ResponseValidationChoice::default(), value_enum)]
    pub validation: ResponseValidationChoice,
    /// Variables will be added to the hurl variables file and the header of each request. Format: `HEADER_KEY=HEADER_VALUE`.
    #[arg(short = 'r', long, value_parser = parse_key_val::<String, String>)]
    pub header_vars: Vec<(String, String)>,
    /// Choose whether, and how to, pass query params.
    #[arg(short = 'q', long, default_value_t = QueryParamChoice::default(), value_enum)]
    pub query_params: QueryParamChoice,
    /// Generate only the operations identified with this options, e.g. `openapi-to-hurl openapi.json -i getProducts -i createProduct`.
    #[arg(short = 'i', long)]
    pub operation_id: Option<Vec<String>>,
    /// Generate only the operations under the given tags, e.g. `openapi-to-hurl openapi.json -t orders -t products`. 
    /// If used with the `operation-id` option the generated files will first be narrowed by tag then by operationId.
    #[arg(short = 't', long)]
    pub tag: Option<Vec<String>>,
    /// How the variables file should be updated.
    #[arg(long, default_value_t = VariablesUpdateStrategy::default(), value_enum)]
    pub variables_file_update: VariablesUpdateStrategy,
    /// How the output should be formatted.
    #[arg(long, default_value_t = Formatting::default(), value_enum)]
    pub formatting: Formatting,
    /// Desired content type of the request.
    /// If the selected content type is not available in the schema or not
    /// supported by this tool the tool will select the first scpecified content type supported by this
    /// tool. If no valid content type is found the tool will use an empty request body.
    #[arg(long, default_value_t = CliContentType::default(), value_enum)]
    pub content_type: CliContentType,
    #[arg(short = 'l', long, default_value_t = LogLevel::default(), value_enum)]
    pub log_level: LogLevel,
    /// Set this to true to silence all logging.
    #[arg(long, default_value_t = false)]
    pub quiet: bool,
    /// How to handle recoverable errors.
    #[arg(long, default_value_t = ErrorHandling::default(), value_enum)]
    pub error_handling: ErrorHandling,
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

