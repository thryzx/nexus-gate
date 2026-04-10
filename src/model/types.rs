use serde::{Deserialize, Serialize};

/// Supported upstream platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Claude,
    OpenAI,
    Gemini,
    Bedrock,
    Azure,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::OpenAI => "openai",
            Self::Gemini => "gemini",
            Self::Bedrock => "bedrock",
            Self::Azure => "azure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Unavailable,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    OAuth,
    ApiKey,
    Bedrock,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_serde() {
        let p = Platform::Claude;
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(s, "\"claude\"");

        let parsed: Platform = serde_json::from_str("\"openai\"").unwrap();
        assert_eq!(parsed, Platform::OpenAI);
    }

    #[test]
    fn platform_as_str() {
        assert_eq!(Platform::Gemini.as_str(), "gemini");
        assert_eq!(Platform::Bedrock.as_str(), "bedrock");
    }
}
