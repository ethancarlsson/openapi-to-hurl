use oas3::Spec;

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

#[derive(PartialEq, Debug)]
pub struct VariableFiles {
    pub files: Vec<VariableFile>,
}

impl VariableFiles {
    pub fn from_spec(spec: &Spec) -> VariableFiles {
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
                    key_vals: vec![("host".to_string(), s.url.clone())],
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::{VariableFile, VariableFiles};

    #[test]
    fn variables_file_from_spec() {
        let spec_path = PathBuf::from_str("test_files/pet_store.json").unwrap();
        let spec = oas3::from_path(spec_path.clone()).unwrap();
        let expected = VariableFiles {
            files: vec![VariableFile {
                name: "petstore.swagger.io_v1".to_string(),
                key_vals: vec![(
                    "host".to_string(),
                    "http://petstore.swagger.io/v1".to_string(),
                )],
            }],
        };

        assert_eq!(expected, VariableFiles::from_spec(&spec));
    }
}
