use crate::{
    cli::{
        Cli, ErrorHandling, Formatting, LogLevel, OutputTo, QueryParamChoice,
        ResponseValidationChoice, VariablesUpdateStrategy,
    },
    variable_files::CustomVariables, content_type::ContentType,
};
use anyhow::anyhow;

#[derive(Default)]
pub enum OutStrategy {
    #[default]
    Console,
    Files(std::path::PathBuf),
}

// V O vs O V
#[derive(Default)]
pub struct Settings {
    pub input: Option<std::path::PathBuf>,
    pub out: OutStrategy,
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
            out: match cli.output_to {
                OutputTo::Console => OutStrategy::Console,
                OutputTo::Files => OutStrategy::Files(match cli.out {
                    Some(f) => f,
                    None => {
                        return Err(anyhow!(
                            "Option `out` is required if `--output-to files` option is selected"
                        ))
                    }
                }),
            },
            validate_response: cli.validate_response,
            query_params_choice: cli.query_params,
            variables_update_strategy: cli.variables_update_strategy,
            custom_variables: CustomVariables {
                headers: cli.header_vars,
            },
            operation_id_selection: cli.select_operation_id,
            tags: cli.tag,
            formatting: cli.formatting,
            content_type: cli.content_type.into(),
            log_level: cli.log_level,
            quiet: cli.quiet,
            error_handling: cli.handle_errors,
        })
    }
}
