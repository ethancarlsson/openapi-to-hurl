use clap::{Parser, ValueEnum};

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
}

pub struct Arguments {
    pub path: std::path::PathBuf,
    pub out: std::path::PathBuf,
    pub validate_response: ResponseValidationChoice,
}

impl Cli {
    pub fn args(self) -> Arguments {
        Arguments { path: self.path, out: self.out, validate_response: self.validate_response }
    }
}
