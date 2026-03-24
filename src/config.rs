use crate::types::BitbucketConfig;
use std::env;

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
    InvalidUrl(String),
    ParseError(String),
    NoAuthMethod,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::NoAuthMethod => write!(f, "No authentication method configured. Set BITBUCKET_API_KEY or BITBUCKET_USERNAME/BITBUCKET_PASSWORD"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_from_env() -> Result<BitbucketConfig, ConfigError> {
    let base_url = env::var("BITBUCKET_BASE_URL")
        .map_err(|_| ConfigError::MissingEnvVar("BITBUCKET_BASE_URL".to_string()))?;
    
    if base_url.is_empty() {
        return Err(ConfigError::InvalidUrl("Base URL cannot be empty".to_string()));
    }
    
    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err(ConfigError::InvalidUrl("Base URL must start with http:// or https://".to_string()));
    }
    
    let verify_ssl = env::var("BITBUCKET_SSL_VERIFY")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    let allow_read = env::var("BITBUCKET_ALLOW_READ")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    let allow_write = env::var("BITBUCKET_ALLOW_WRITE")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    let allow_delete = env::var("BITBUCKET_ALLOW_DELETE")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    use crate::auth::AuthMethod;
    use secrecy::SecretString;
    
    // Determine authentication method with API key priority
    let auth = {
        let api_key = env::var("BITBUCKET_API_KEY").ok();
        if let Some(key) = api_key {
            if !key.is_empty() {
                // Use Bearer auth with API key
                AuthMethod::Bearer(SecretString::from(key))
            } else {
                // API key is empty, fall back to Basic auth
                let username = env::var("BITBUCKET_USERNAME")
                    .map_err(|_| ConfigError::MissingEnvVar("BITBUCKET_USERNAME".to_string()))?;
                
                if username.is_empty() {
                    return Err(ConfigError::ParseError("Username cannot be empty".to_string()));
                }
                
                let password = env::var("BITBUCKET_PASSWORD")
                    .map_err(|_| ConfigError::MissingEnvVar("BITBUCKET_PASSWORD".to_string()))?;
                
                if password.is_empty() {
                    return Err(ConfigError::ParseError("Password cannot be empty".to_string()));
                }
                
                AuthMethod::Basic {
                    username,
                    password: SecretString::from(password),
                }
            }
        } else {
            // No API key set, use Basic auth
            let username = env::var("BITBUCKET_USERNAME")
                .map_err(|_| ConfigError::MissingEnvVar("BITBUCKET_USERNAME".to_string()))?;
            
            if username.is_empty() {
                return Err(ConfigError::ParseError("Username cannot be empty".to_string()));
            }
            
            let password = env::var("BITBUCKET_PASSWORD")
                .map_err(|_| ConfigError::MissingEnvVar("BITBUCKET_PASSWORD".to_string()))?;
            
            if password.is_empty() {
                return Err(ConfigError::ParseError("Password cannot be empty".to_string()));
            }
            
            AuthMethod::Basic {
                username,
                password: SecretString::from(password),
            }
        }
    };
    
    Ok(BitbucketConfig {
        base_url,
        auth,
        verify_ssl,
        allow_read,
        allow_write,
        allow_delete,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthMethod;
    use serial_test::serial;
    
    #[test]
    #[serial(config_env)]
    fn test_parse_valid_config() {
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.base_url, "https://bitbucket.example.com");
        assert_eq!(config.verify_ssl, true);
        assert_eq!(config.allow_read, true);
        assert_eq!(config.allow_write, true);
        assert_eq!(config.allow_delete, true);
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    #[serial(config_env)]
    fn test_missing_base_url() {
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_err());
        
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    #[serial(config_env)]
    fn test_invalid_url_scheme() {
        std::env::set_var("BITBUCKET_BASE_URL", "ftp://bitbucket.example.com");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_err());
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    #[serial(config_env)]
    fn test_api_key_only_uses_bearer_auth() {
        // Clean up all auth-related env vars first
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
        
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_API_KEY", "test-api-key-12345");
        
        let config = load_from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(matches!(config.auth, AuthMethod::Bearer(_)));
        
        // Clean up all env vars at the end
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    #[serial(config_env)]
    fn test_basic_auth_only_uses_basic() {
        // Clean up all auth-related env vars first
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
        
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(matches!(config.auth, AuthMethod::Basic { .. }));
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    #[serial(config_env)]
    fn test_api_key_has_priority_over_basic_auth() {
        // Clean up all auth-related env vars first
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
        
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_API_KEY", "test-api-key-12345");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(matches!(config.auth, AuthMethod::Bearer(_)));
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    #[serial(config_env)]
    fn test_empty_api_key_falls_back_to_basic_auth() {
        // Clean up all auth-related env vars first
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
        
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_API_KEY", "");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(matches!(config.auth, AuthMethod::Basic { .. }));
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
    fn test_no_auth_method_returns_error() {
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::remove_var("BITBUCKET_API_KEY");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
        
        let config = load_from_env();
        assert!(config.is_err());
        assert!(matches!(config.unwrap_err(), ConfigError::MissingEnvVar(_)));
        
        std::env::remove_var("BITBUCKET_BASE_URL");
    }
}
