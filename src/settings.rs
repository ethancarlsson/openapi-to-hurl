use crate::{
    cli::{
        Cli, ErrorHandling, Formatting, LogLevel, QueryParamChoice, ResponseValidationChoice,
        VariablesUpdateStrategy,
    },
    content_type::ContentType,
    variable_files::CustomVariables,
};

#[derive(Default)]
pub struct Settings {
    pub input: Option<std::path::PathBuf>,
    pub out_dir: Option<std::path::PathBuf>,
    pub validate_response: ResponseValidationChoice,
    pub query_params_choice: QueryParamChoice,
    pub custom_variables: CustomVariables,
    pub variables_update_strategy: VariablesUpdateStrategy,
    pub operation_id_selection: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub formatting: Formatting,
    pub content_type: ContentType,
    pub log_level: LogLevel,
    pub quiet: bool,
    pub error_handling: ErrorHandling,
}

impl TryFrom<Cli> for Settings {
    type Error = anyhow::Error;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        Ok(Self {
            input: cli.input,
            out_dir: cli.out_dir,
            validate_response: cli.validation,
            query_params_choice: cli.query_params,
            variables_update_strategy: cli.variables_file_update,
            custom_variables: CustomVariables {
                headers: cli.header_vars,
            },
            operation_id_selection: cli.operation_id,
            tags: cli.tag,
            formatting: cli.formatting,
            content_type: cli.content_type.into(),
            log_level: cli.log_level,
            quiet: cli.quiet,
            error_handling: cli.error_handling,
        })
    }
}
