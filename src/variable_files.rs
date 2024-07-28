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

    pub fn from_string(name: String, str_file: String) -> Self {
        VariableFile {
            name,
            key_vals: str_file
                .split("\n")
                .map(|s| {
                    let mut kv = s.split("=");
                    (
                        kv.next().unwrap_or("").to_string(),
                        kv.next().unwrap_or("").to_string(),
                    )
                })
                .collect::<Vec<(String, String)>>(),
        }
    }

    pub fn empty(name: String) -> Self {
        VariableFile {
            name,
            key_vals: vec![],
        }
    }

    pub fn merge(self, other: VariableFile) -> Self {
        if self.name != other.name {
            return self;
        }

        let other_keys = other
            .key_vals
            .iter()
            .map(|kv| kv.0.clone())
            .collect::<Vec<String>>();

        // Filter out all keys in the new file
        let mut new_kvs = self
            .key_vals
            .iter()
            .filter_map(|kv| {
                if !other_keys.contains(&kv.0) {
                    Some(kv.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<(String, String)>>();

        // Extend the file with all the new values
        new_kvs.extend(other.key_vals.iter().map(|kv| kv.clone()));

        Self {
            name: self.name,
            key_vals: new_kvs,
        }
    }
}

#[derive(Default)]
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
                    ("Authorization".to_string(), "Bearer test".to_string()),
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

    #[test]
    fn variables_file_merge_with_differently_named_files_doesnt_merge() {
        let file1 = VariableFile::from_string("test1".to_string(), "hello=world1".to_string());
        let file2 = VariableFile::from_string("test2".to_string(), "hello=world2".to_string());

        assert_eq!(
            "hello=world1".to_string(),
            file1.merge(file2).get_contents()
        );
    }

    #[test]
    fn variables_file_merge_with_same_key_overrides_old_key_value() {
        let file1 =
            VariableFile::from_string("test".to_string(), "hello=world1\ntest1=test1".to_string());
        let file2 = VariableFile::from_string("test".to_string(), "hello=world2".to_string());

        assert_eq!(
            "test1=test1\nhello=world2".to_string(),
            file1.merge(file2).get_contents()
        );
    }

    #[test]
    fn variables_file_merge_with_new_key_appends_key() {
        let file1 =
            VariableFile::from_string("test".to_string(), "hello=world1\ntest1=test1".to_string());
        let file2 = VariableFile::from_string("test".to_string(), "hey=world2".to_string());

        assert_eq!(
            "hello=world1\ntest1=test1\nhey=world2".to_string(),
            file1.merge(file2).get_contents()
        );
    }

    #[test]
    fn variables_file_merge_with_no_new() {
        let file1 =
            VariableFile::from_string("test".to_string(), "hello=world1\ntest1=test1".to_string());
        let file2 = VariableFile::empty("test".to_string());

        assert_eq!(
            "hello=world1\ntest1=test1".to_string(),
            file1.merge(file2).get_contents()
        );
    }

    #[test]
    fn variables_file_merge_with_no_old() {
        let file1 = VariableFile::empty("test".to_string());
        let file2 =
            VariableFile::from_string("test".to_string(), "hello=world1\ntest1=test1".to_string());

        assert_eq!(
            "hello=world1\ntest1=test1".to_string(),
            file1.merge(file2).get_contents()
        );
    }
}
