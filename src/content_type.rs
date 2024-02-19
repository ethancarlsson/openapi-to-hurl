use crate::cli::CliContentType;

const PLAIN_TEXT: &str = "text";
const JSON: &str = "json";

#[derive(Clone, Default)]
pub enum ContentType {
    Text,
    #[default]
    Json,
}

impl From<CliContentType> for ContentType {
    fn from(value: CliContentType) -> Self {
        match value {
            CliContentType::Text => Self::Text,
            CliContentType::Json => Self::Json,
        }
    }
}

impl ContentType {
    pub fn matches_string(&self, str: &String) -> bool {
        match self {
            ContentType::Text => str.contains(PLAIN_TEXT),
            ContentType::Json => str.contains(JSON),
        }
    }

    pub fn is_supported(content_type: &String) -> bool {
        ContentType::supported_types()
            .iter()
            .find(|ct| content_type.contains(&ct.to_string()))
            .is_some()
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            ContentType::Text => PLAIN_TEXT,
            ContentType::Json => JSON,
        }
    }

    pub fn supported_types() -> Vec<String> {
        vec![PLAIN_TEXT.to_string(), JSON.to_string()]
    }

    pub fn from_string(content_type: &String) -> Result<Self, String> {
        if content_type.contains(PLAIN_TEXT) {
            Ok(Self::Text)
        } else if content_type.contains(JSON) {
            Ok(Self::Json)
        } else {
            Err("Unsupported content type".to_string())
        }
    }
}
