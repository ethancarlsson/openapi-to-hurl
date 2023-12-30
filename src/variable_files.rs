use oas3::{spec::Server, Spec};

#[derive(PartialEq, Debug)]
pub struct VariableFile {
    pub name: String,
    pub key_vals: Vec<(String, String)>,
}

impl VariableFile {
    pub fn get_contents(self) -> String {
        self.key_vals
            .iter()
            .map(|kv| format!("{}={}", kv.0, kv.1))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

pub struct CustomVariables {
    pub headers: Vec<(String, String)>,
}

#[derive(PartialEq, Debug)]
pub struct VariableFiles {
    pub files: Vec<VariableFile>,
}

impl VariableFiles {
    pub fn from_spec(spec: &Spec, custom_variables: CustomVariables) -> VariableFiles {
        VariableFiles {
            files: spec
                .servers
                .iter()
                .map(|s| VariableFile {
                    name: s
                        .url
                        .replace("https://", "")
                        .replace("http://", "")
                        .replace("/", "_"),
                    key_vals: Self::build_key_vals(&s, &custom_variables),
                })
                .collect(),
        }
    }

    fn build_key_vals(
        server: &Server,
        custom_variables: &CustomVariables,
    ) -> Vec<(String, String)> {
        let mut key_vals = vec![("host".to_string(), server.url.clone())];
        key_vals.extend(
            custom_variables
                .headers
                .iter()
                .map(|h| (h.0.clone(), h.1.clone())),
        );

        key_vals
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::variable_files::CustomVariables;

    use super::{VariableFile, VariableFiles};

    #[test]
    fn variables_file_from_spec() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();
        let expected = VariableFiles {
            files: vec![VariableFile {
                name: "petstore.swagger.io_v1".to_string(),
                key_vals: vec![
                    (
                        "host".to_string(),
                        "http://petstore.swagger.io/v1".to_string(),
                    ),
                    (
                        "Authorization".to_string(),
                        "Bearer test".to_string(),
                    ),
                ],
            }],
        };

        assert_eq!(
            expected,
            VariableFiles::from_spec(
                &spec,
                CustomVariables {
                    headers: vec![("Authorization".to_string(), "Bearer test".to_string())]
                }
            )
        );
    }
}
