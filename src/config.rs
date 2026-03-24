use crate::types::BitbucketConfig;
use std::env;

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
    InvalidUrl(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
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
    
    Ok(BitbucketConfig {
        base_url,
        username,
        password,
        verify_ssl,
        allow_read,
        allow_write,
        allow_delete,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_config() {
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        
        let config = load_from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.base_url, "https://bitbucket.example.com");
        assert_eq!(config.username, "testuser");
        assert_eq!(config.password, "testpass");
        assert_eq!(config.verify_ssl, true);
        assert_eq!(config.allow_read, true);
        assert_eq!(config.allow_write, true);
        assert_eq!(config.allow_delete, true);
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
    }
    
    #[test]
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
    fn test_ssl_verify_defaults() {
        std::env::set_var("BITBUCKET_BASE_URL", "https://bitbucket.example.com");
        std::env::set_var("BITBUCKET_USERNAME", "testuser");
        std::env::set_var("BITBUCKET_PASSWORD", "testpass");
        std::env::remove_var("BITBUCKET_SSL_VERIFY");
        
        let config = load_from_env().unwrap();
        assert_eq!(config.verify_ssl, true);
        
        std::env::set_var("BITBUCKET_SSL_VERIFY", "false");
        let config = load_from_env().unwrap();
        assert_eq!(config.verify_ssl, false);
        
        std::env::remove_var("BITBUCKET_BASE_URL");
        std::env::remove_var("BITBUCKET_USERNAME");
        std::env::remove_var("BITBUCKET_PASSWORD");
        std::env::remove_var("BITBUCKET_SSL_VERIFY");
    }
}
