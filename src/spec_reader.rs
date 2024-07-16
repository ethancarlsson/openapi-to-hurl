use anyhow::anyhow;
use std::io::Read;

use anyhow::{Context, Result};
use oas3::Spec;

const GENERIC_ERROR_MESSAGE: &str = "Invalid Open API 3.1 Specification or file I/O error.";
const NOT_MATCHED_UNTAGGED_ENUM_MSG: &str =
    "data did not match any variant of untagged enum ObjectOrReference";

pub fn from_path(p: std::path::PathBuf) -> Result<Spec, anyhow::Error> {
    match oas3::from_path(p.clone()).with_context(|| GENERIC_ERROR_MESSAGE) {
        Ok(s) => Ok(s),
        Err(e) => {
            let error_message = e.root_cause().to_string();
            if error_message.contains(NOT_MATCHED_UNTAGGED_ENUM_MSG) {
                Err(anyhow!(format!("Specification error at: {}. Please ensure the object at that location has the correct structure and all types match OpenAPI Specification v3.1.0, earlier or later versions are not supported by this tool.", error_message.replace(": data did not match any variant of untagged enum ObjectOrReference at", "")))).with_context(|| GENERIC_ERROR_MESSAGE)
            } else {
                Err(e)
            }
        }
    }
}

pub fn from_reader<R>(p: R) -> Result<Spec, anyhow::Error>
where
    R: Read,
{
    match oas3::from_reader(p).with_context(|| GENERIC_ERROR_MESSAGE) {
        Ok(s) => Ok(s),
        Err(e) => {
            let error_message = e.root_cause().to_string();
            if error_message.contains(NOT_MATCHED_UNTAGGED_ENUM_MSG) {
                Err(anyhow!(format!("Specification error at: {}. Please ensure the object at that location has the correct structure and all types match OpenAPI Specification v3.1.0, earlier or later versions are not supported by this tool.", error_message.replace(": data did not match any variant of untagged enum ObjectOrReference at", "")))).with_context(|| GENERIC_ERROR_MESSAGE)
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::from_path;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_path_with_valid_spec_returns_spec() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = from_path(spec_path);

        assert_eq!(true, spec.is_ok())
    }

    #[test]
    fn from_path_with_invalid_spec_returns_error_with_custom_message() {
        let spec_path = PathBuf::from_str("test_files/pet_store_invalid.json").unwrap();
        let spec = from_path(spec_path);

        assert_eq!(true, spec.is_err());

        assert_eq!("Specification error at: paths./pets.get.parameters line 24 column 23. Please ensure the object at that location has the correct structure and all types match OpenAPI Specification v3.1.0, earlier or later versions are not supported by this tool.", spec.unwrap_err().root_cause().to_string());
    }

    #[test]
    fn from_path_with_file_not_found_returns_error() {
        let spec_path = PathBuf::from_str("test_files/pet_store_invalid_not_found.json").unwrap();
        let spec = from_path(spec_path);

        assert_eq!(true, spec.is_err());
        let err = spec.unwrap_err();
        assert_eq!(
            "No such file or directory (os error 2)",
            err.root_cause().to_string()
        )
    }
}
