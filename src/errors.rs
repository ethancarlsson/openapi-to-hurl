use oas3::spec::RefError;

pub enum OperationError {
    Ref(Option<String>, RefError),
}

impl std::fmt::Display for OperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationError::Ref(operation_id, ref_error) => write!(
                f,
                "{}{ref_error}",
                match operation_id {
                    Some(id) => format!("{}: ", id),
                    None => "".to_string(),
                }
            ),
        }
    }
}
