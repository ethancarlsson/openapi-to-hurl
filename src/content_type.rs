use clap::ValueEnum;

const PLAIN_TEXT: &str = "text/plain";
const JSON: &str = "json";

#[derive(ValueEnum, Clone, Default)]
pub enum ContentType {
    PlainText,
    #[default]
    Json,
}


impl ContentType {
    pub fn matches_string(&self, str: &String) -> bool {
        match self {
            ContentType::PlainText => str.contains(PLAIN_TEXT),
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
            ContentType::PlainText => PLAIN_TEXT,
            ContentType::Json => JSON,
        }
    }

    pub fn supported_types() -> Vec<String> {
        vec![PLAIN_TEXT.to_string(), JSON.to_string()]
    }

    pub fn from_string(content_type: &String) -> Result<Self, String> {
        if content_type.contains(PLAIN_TEXT) {
            Ok(Self::PlainText)
        } else if content_type.contains(JSON) {
            Ok(Self::Json)
        } else {
            Err("Unsupported content type".to_string())
        }
    }
}
