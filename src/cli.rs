use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
pub struct Cli {
    /// The path to the openapi specification
    pub path: std::path::PathBuf,
    /// Directory where the hurl files will be created
    pub out: std::path::PathBuf,
}
