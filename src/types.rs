use secrecy::ExposeSecret;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::auth::AuthMethod;

#[derive(Debug, Clone)]
pub struct BitbucketConfig {
    pub base_url: String,
    pub auth: AuthMethod,
    pub verify_ssl: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_delete: bool,
}

// Custom serialization for BitbucketConfig to support AuthMethod
impl Serialize for BitbucketConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("BitbucketConfig", 7)?;
        state.serialize_field("base_url", &self.base_url)?;
        match &self.auth {
            AuthMethod::None => {
                state.serialize_field("username", "")?;
                state.serialize_field("password", "")?;
            }
            AuthMethod::Basic { username, password } => {
                state.serialize_field("username", username)?;
                state.serialize_field("password", password.expose_secret())?;
            }
            AuthMethod::Bearer(token) => {
                state.serialize_field("username", "")?;
                state.serialize_field("password", token.expose_secret())?;
            }
        }
        state.serialize_field("verify_ssl", &self.verify_ssl)?;
        state.serialize_field("allow_read", &self.allow_read)?;
        state.serialize_field("allow_write", &self.allow_write)?;
        state.serialize_field("allow_delete", &self.allow_delete)?;
        state.end()
    }
}

// Custom deserialization for BitbucketConfig to support AuthMethod
impl<'de> Deserialize<'de> for BitbucketConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(serde::Deserialize)]
        struct BitbucketConfigHelper {
            base_url: String,
            username: String,
            password: String,
            verify_ssl: bool,
            allow_read: bool,
            allow_write: bool,
            allow_delete: bool,
        }

        struct BitbucketConfigVisitor;

        impl<'de> Visitor<'de> for BitbucketConfigVisitor {
            type Value = BitbucketConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BitbucketConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<BitbucketConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                let helper = BitbucketConfigHelper::deserialize(
                    de::value::MapAccessDeserializer::new(&mut map)
                )?;
                
                let auth = if helper.username.is_empty() && helper.password.is_empty() {
                    AuthMethod::None
                } else {
                    AuthMethod::Basic {
                        username: helper.username,
                        password: secrecy::SecretString::from(helper.password),
                    }
                };

                Ok(BitbucketConfig {
                    base_url: helper.base_url,
                    auth,
                    verify_ssl: helper.verify_ssl,
                    allow_read: helper.allow_read,
                    allow_write: helper.allow_write,
                    allow_delete: helper.allow_delete,
                })
            }
        }

        deserializer.deserialize_struct("BitbucketConfig", &[
            "base_url", "username", "password", "verify_ssl",
            "allow_read", "allow_write", "allow_delete"
        ], BitbucketConfigVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub values: Vec<T>,
    #[serde(rename = "nextPageStart")]
    pub next_page_start: Option<u32>,
    #[serde(rename = "isLastPage")]
    pub is_last_page: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthMethod;
    use secrecy::SecretString;

    #[test]
    fn test_config_serialization_basic_auth() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::Basic {
                username: "testuser".to_string(),
                password: SecretString::from("testpass".to_string()),
            },
            verify_ssl: true,
            allow_read: true,
            allow_write: false,
            allow_delete: false,
        };

        let serialized = serde_json::to_string(&config).expect("Failed to serialize");
        assert!(serialized.contains("testuser"));
        assert!(serialized.contains("testpass"));
        assert!(serialized.contains("bitbucket.example.com"));
    }

    #[test]
    fn test_config_serialization_bearer_auth() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::Bearer(SecretString::from("bearer-token".to_string())),
            verify_ssl: true,
            allow_read: true,
            allow_write: false,
            allow_delete: false,
        };

        let serialized = serde_json::to_string(&config).expect("Failed to serialize");
        // Bearer auth serializes password field with token, username is empty
        assert!(serialized.contains("bearer-token"));
    }

    #[test]
    fn test_config_serialization_none_auth() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::None,
            verify_ssl: true,
            allow_read: true,
            allow_write: false,
            allow_delete: false,
        };

        let serialized = serde_json::to_string(&config).expect("Failed to serialize");
        // None auth serializes both username and password as empty
        assert!(serialized.contains("\"username\":\"\""));
        assert!(serialized.contains("\"password\":\"\""));
    }

    #[test]
    fn test_config_deserialization_basic_auth() {
        let json = r#"{
            "base_url": "https://bitbucket.example.com",
            "username": "testuser",
            "password": "testpass",
            "verify_ssl": true,
            "allow_read": true,
            "allow_write": false,
            "allow_delete": false
        }"#;

        let config: BitbucketConfig = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(config.base_url, "https://bitbucket.example.com");
        assert!(matches!(config.auth, AuthMethod::Basic { .. }));
        assert!(config.verify_ssl);
        assert!(config.allow_read);
        assert!(!config.allow_write);
        assert!(!config.allow_delete);
    }

    #[test]
    fn test_config_deserialization_none_auth() {
        let json = r#"{
            "base_url": "https://bitbucket.example.com",
            "username": "",
            "password": "",
            "verify_ssl": true,
            "allow_read": true,
            "allow_write": false,
            "allow_delete": false
        }"#;

        let config: BitbucketConfig = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(config.base_url, "https://bitbucket.example.com");
        assert!(matches!(config.auth, AuthMethod::None));
    }

    #[test]
    fn test_config_round_trip_serialization() {
        let original = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::Basic {
                username: "roundtrip".to_string(),
                password: SecretString::from("secret123".to_string()),
            },
            verify_ssl: false,
            allow_read: true,
            allow_write: true,
            allow_delete: false,
        };

        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: BitbucketConfig = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(original.base_url, deserialized.base_url);
        assert_eq!(original.verify_ssl, deserialized.verify_ssl);
        assert_eq!(original.allow_read, deserialized.allow_read);
        assert_eq!(original.allow_write, deserialized.allow_write);
        assert_eq!(original.allow_delete, deserialized.allow_delete);
        
        // Verify auth method is preserved
        assert!(matches!(deserialized.auth, AuthMethod::Basic { .. }));
    }
}
