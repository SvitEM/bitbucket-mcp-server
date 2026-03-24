pub mod error;
pub mod projects;
pub mod repositories;
pub mod branches;
pub mod pull_requests;
pub mod commits;
pub mod files;

use crate::types::BitbucketConfig;
use error::ApiError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

/// HTTP client for Bitbucket Server API 1.0
pub struct BitbucketClient {
    client: reqwest::Client,
    base_url: String,
    auth_header: String,
}

impl BitbucketClient {
    /// Creates a new Bitbucket API client with the given configuration
    pub fn new(config: BitbucketConfig) -> Result<Self, ApiError> {
        let auth_header = config.auth.to_header_value();
        
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()?;
        
        Ok(Self {
            client,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            auth_header,
        })
    }
    
    /// Builds a full API URL from a relative path
    pub fn build_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/rest/api/1.0/{}", self.base_url, path)
    }
    
    /// Creates default headers with authentication
    pub fn default_headers(&self) -> Result<HeaderMap, ApiError> {
        let mut headers = HeaderMap::new();
        let auth_value = HeaderValue::from_str(&self.auth_header)
            .map_err(|e| ApiError::AuthError(format!("Invalid auth header: {}", e)))?;
        headers.insert(AUTHORIZATION, auth_value);
        Ok(headers)
    }
    
    /// Builds a paginated URL with start and limit parameters
    pub fn build_paginated_url(&self, path: &str, start: u32, limit: u32) -> String {
        let base_url = self.build_url(path);
        format!("{}?start={}&limit={}", base_url, start, limit)
    }
    
    /// Extracts nextPageStart from a Bitbucket API response
    pub fn extract_next_page_start(response: &serde_json::Value) -> Option<u32> {
        response.get("nextPageStart")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
    }
    
    /// Checks if the response indicates the last page
    pub fn is_last_page(response: &serde_json::Value) -> bool {
        response.get("isLastPage")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthMethod;
    use secrecy::SecretString;
    
    #[test]
    fn test_build_url() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::Basic {
                username: "test".to_string(),
                password: SecretString::from("test".to_string()),
            },
            verify_ssl: true,
            allow_read: true,
            allow_write: true,
            allow_delete: true,
        };
        
        let client = BitbucketClient::new(config).unwrap();
        
        assert_eq!(
            client.build_url("/projects"),
            "https://bitbucket.example.com/rest/api/1.0/projects"
        );
        
        assert_eq!(
            client.build_url("projects"),
            "https://bitbucket.example.com/rest/api/1.0/projects"
        );
    }
    
    #[test]
    fn test_build_url_trailing_slash() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com/".to_string(),
            auth: AuthMethod::Basic {
                username: "test".to_string(),
                password: SecretString::from("test".to_string()),
            },
            verify_ssl: true,
            allow_read: true,
            allow_write: true,
            allow_delete: true,
        };
        
        let client = BitbucketClient::new(config).unwrap();
        assert_eq!(
            client.build_url("projects"),
            "https://bitbucket.example.com/rest/api/1.0/projects"
        );
    }
    
    #[test]
    fn test_client_creation() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::Basic {
                username: "testuser".to_string(),
                password: SecretString::from("testpass".to_string()),
            },
            verify_ssl: true,
            allow_read: true,
            allow_write: true,
            allow_delete: true,
        };
        
        let client = BitbucketClient::new(config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_paginated_url() {
        let config = BitbucketConfig {
            base_url: "https://bitbucket.example.com".to_string(),
            auth: AuthMethod::Basic {
                username: "test".to_string(),
                password: SecretString::from("test".to_string()),
            },
            verify_ssl: true,
            allow_read: true,
            allow_write: true,
            allow_delete: true,
        };
        
        let client = BitbucketClient::new(config).unwrap();
        let url = client.build_paginated_url("projects", 25, 50);
        assert_eq!(url, "https://bitbucket.example.com/rest/api/1.0/projects?start=25&limit=50");
    }
    
    #[test]
    fn test_extract_next_page_start() {
        let response = serde_json::json!({
            "size": 25,
            "limit": 25,
            "isLastPage": false,
            "nextPageStart": 25,
            "values": []
        });
        
        let next_start = BitbucketClient::extract_next_page_start(&response);
        assert_eq!(next_start, Some(25));
    }
    
    #[test]
    fn test_is_last_page() {
        let response_last = serde_json::json!({
            "size": 10,
            "limit": 25,
            "isLastPage": true,
            "values": []
        });
        
        let response_not_last = serde_json::json!({
            "size": 25,
            "limit": 25,
            "isLastPage": false,
            "nextPageStart": 25,
            "values": []
        });
        
        assert!(BitbucketClient::is_last_page(&response_last));
        assert!(!BitbucketClient::is_last_page(&response_not_last));
    }
}
