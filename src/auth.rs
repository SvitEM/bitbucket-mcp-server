use secrecy::{ExposeSecret, SecretString};

/// Authentication method for Bitbucket API requests.
/// Supports Basic authentication and Bearer token authentication.
#[derive(Clone)]
pub enum AuthMethod {
    /// No authentication (use with caution)
    None,
    /// HTTP Basic authentication with username and password/PAT
    Basic {
        username: String,
        password: SecretString,
    },
    /// Bearer token authentication
    Bearer(SecretString),
}

impl std::fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::None => write!(f, "AuthMethod::None"),
            AuthMethod::Basic { username, .. } => {
                write!(f, "AuthMethod::Basic {{ username: {:?}, password: [hidden] }}", username)
            }
            AuthMethod::Bearer(_) => {
                write!(f, "AuthMethod::Bearer([hidden])")
            }
        }
    }
}

impl AuthMethod {
    /// Generate the Authorization header value for this auth method.
    ///
    /// # Returns
    /// - `None`: Returns empty string (no Authorization header needed)
    /// - `Basic`: Returns "Basic {base64(username:password)}"
    /// - `Bearer`: Returns "Bearer {token}"
    pub fn to_header_value(&self) -> String {
        match self {
            AuthMethod::None => String::new(),
            AuthMethod::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password.expose_secret());
                let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &credentials);
                format!("Basic {}", encoded)
            }
            AuthMethod::Bearer(token) => {
                format!("Bearer {}", token.expose_secret())
            }
        }
    }

    /// Check if authentication is configured (not None).
    ///
    /// # Returns
    /// - `true` if Basic or Bearer variant
    /// - `false` if None variant
    pub fn is_configured(&self) -> bool {
        !matches!(self, AuthMethod::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_is_configured() {
        let auth = AuthMethod::None;
        assert!(!auth.is_configured());
        assert_eq!(auth.to_header_value(), "");
    }

    #[test]
    fn test_basic_is_configured() {
        let auth = AuthMethod::Basic {
            username: "test_user".to_string(),
            password: SecretString::from("test_password".to_string()),
        };
        assert!(auth.is_configured());
        assert!(auth.to_header_value().starts_with("Basic "));
    }

    #[test]
    fn test_bearer_is_configured() {
        let auth = AuthMethod::Bearer(SecretString::from("test_token".to_string()));
        assert!(auth.is_configured());
        assert_eq!(auth.to_header_value(), "Bearer test_token");
    }

    #[test]
    fn test_debug_hides_secrets() {
        let basic = AuthMethod::Basic {
            username: "user".to_string(),
            password: SecretString::from("secret".to_string()),
        };
        let debug_str = format!("{:?}", basic);
        assert!(debug_str.contains("[hidden]"));
        assert!(!debug_str.contains("secret"));

        let bearer = AuthMethod::Bearer(SecretString::from("token".to_string()));
        let debug_str = format!("{:?}", bearer);
        assert!(debug_str.contains("[hidden]"));
        assert!(!debug_str.contains("token"));
    }

    #[test]
    fn test_none_to_header_returns_empty() {
        let auth = AuthMethod::None;
        assert_eq!(auth.to_header_value(), "");
    }

    #[test]
    fn test_basic_with_empty_username() {
        let auth = AuthMethod::Basic {
            username: "".to_string(),
            password: SecretString::from("password".to_string()),
        };
        let header = auth.to_header_value();
        assert!(header.starts_with("Basic "));
        // Empty username should still produce valid base64 encoding
        assert_eq!(header, "Basic OnBhc3N3b3Jk");
    }

    #[test]
    fn test_basic_with_special_characters() {
        let auth = AuthMethod::Basic {
            username: "user@company.com".to_string(),
            password: SecretString::from("p@ss:word!".to_string()),
        };
        let header = auth.to_header_value();
        assert!(header.starts_with("Basic "));
        // Verify it encodes correctly with special chars
        let expected_creds = "user@company.com:p@ss:word!";
        let expected_encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, expected_creds);
        assert_eq!(header, format!("Basic {}", expected_encoded));
    }

    #[test]
    fn test_bearer_with_special_characters() {
        let auth = AuthMethod::Bearer(SecretString::from("token-with_special.chars".to_string()));
        let header = auth.to_header_value();
        assert_eq!(header, "Bearer token-with_special.chars");
    }

    #[test]
    fn test_bearer_with_empty_token() {
        let auth = AuthMethod::Bearer(SecretString::from("".to_string()));
        let header = auth.to_header_value();
        assert_eq!(header, "Bearer ");
    }

    #[test]
    fn test_is_configured_all_variants() {
        // None should return false
        assert!(!AuthMethod::None.is_configured());
        
        // Basic should return true even with empty values
        let basic_empty = AuthMethod::Basic {
            username: "".to_string(),
            password: SecretString::from("".to_string()),
        };
        assert!(basic_empty.is_configured());
        
        // Bearer should return true even with empty token
        let bearer_empty = AuthMethod::Bearer(SecretString::from("".to_string()));
        assert!(bearer_empty.is_configured());
    }

    #[test]
    fn test_debug_none_variant() {
        let auth = AuthMethod::None;
        let debug_str = format!("{:?}", auth);
        assert_eq!(debug_str, "AuthMethod::None");
    }
}
