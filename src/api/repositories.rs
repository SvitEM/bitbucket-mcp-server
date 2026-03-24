use serde::{Deserialize, Serialize};
use crate::api::error::ApiError;
use crate::api::BitbucketClient;
use crate::api::projects::Project;
use crate::types::PaginatedResponse;

/// Represents a clone URL for a repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloneLink {
    /// The clone URL (e.g., "https://..." or "ssh://...")
    pub href: String,
    /// The protocol name (e.g., "http", "ssh")
    pub name: String,
}

/// Represents repository links including clone URLs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryLinks {
    /// Array of clone URLs (HTTP, SSH, etc.)
    #[serde(default)]
    pub clone: Vec<CloneLink>,
}

/// Represents a Bitbucket Server repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    /// Unique repository slug (URL-safe identifier)
    pub slug: String,
    /// Human-readable repository name
    pub name: String,
    /// Optional repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The project this repository belongs to
    pub project: Project,
    /// Whether the repository is publicly accessible
    #[serde(default)]
    pub public: bool,
    /// Repository links including clone URLs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<RepositoryLinks>,
}

impl Repository {
    /// Gets the HTTP clone URL if available
    pub fn http_clone_url(&self) -> Option<String> {
        self.links.as_ref()
            .and_then(|links| links.clone.iter()
                .find(|link| link.name == "http")
                .map(|link| link.href.clone()))
    }
    
    /// Gets the SSH clone URL if available
    pub fn ssh_clone_url(&self) -> Option<String> {
        self.links.as_ref()
            .and_then(|links| links.clone.iter()
                .find(|link| link.name == "ssh")
                .map(|link| link.href.clone()))
    }
}

impl BitbucketClient {
    /// Lists repositories in a project with pagination
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `start` - Starting index for pagination (0-based)
    /// * `limit` - Maximum number of repositories to return
    /// 
    /// # Returns
    /// A paginated response containing repositories
    pub async fn list_repos(&self, project_key: &str, start: u32, limit: u32) -> Result<PaginatedResponse<Repository>, ApiError> {
        let path = format!("projects/{}/repos", project_key);
        let url = self.build_paginated_url(&path, start, limit);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<Repository> = response.json().await?;
        Ok(json)
    }
    
    /// Gets a specific repository by project key and repository slug
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// 
    /// # Returns
    /// The requested repository
    pub async fn get_repo(&self, project_key: &str, repo_slug: &str) -> Result<Repository, ApiError> {
        let url = self.build_url(&format!("projects/{}/repos/{}", project_key, repo_slug));
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let repository: Repository = response.json().await?;
        Ok(repository)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_repository_deserialization() {
        let json = json!({
            "slug": "my-repo",
            "name": "My Repository",
            "description": "A test repository",
            "public": false,
            "project": {
                "key": "PROJ",
                "name": "My Project",
                "type": "NORMAL",
                "public": false
            },
            "links": {
                "clone": [
                    {
                        "href": "https://bitbucket.example.com/scm/proj/my-repo.git",
                        "name": "http"
                    },
                    {
                        "href": "ssh://git@bitbucket.example.com:7999/proj/my-repo.git",
                        "name": "ssh"
                    }
                ]
            }
        });
        
        let repo: Repository = serde_json::from_value(json).unwrap();
        assert_eq!(repo.slug, "my-repo");
        assert_eq!(repo.name, "My Repository");
        assert_eq!(repo.description, Some("A test repository".to_string()));
        assert_eq!(repo.public, false);
        assert_eq!(repo.project.key, "PROJ");
        assert_eq!(repo.project.name, "My Project");
        
        let http_url = repo.http_clone_url().unwrap();
        assert_eq!(http_url, "https://bitbucket.example.com/scm/proj/my-repo.git");
        
        let ssh_url = repo.ssh_clone_url().unwrap();
        assert_eq!(ssh_url, "ssh://git@bitbucket.example.com:7999/proj/my-repo.git");
    }
    
    #[test]
    fn test_repository_deserialization_minimal() {
        let json = json!({
            "slug": "test-repo",
            "name": "Test Repository",
            "project": {
                "key": "TEST",
                "name": "Test Project",
                "type": "NORMAL"
            }
        });
        
        let repo: Repository = serde_json::from_value(json).unwrap();
        assert_eq!(repo.slug, "test-repo");
        assert_eq!(repo.name, "Test Repository");
        assert_eq!(repo.description, None);
        assert_eq!(repo.public, false);
        assert_eq!(repo.project.key, "TEST");
        assert!(repo.links.is_none());
        assert!(repo.http_clone_url().is_none());
        assert!(repo.ssh_clone_url().is_none());
    }
    
    #[test]
    fn test_paginated_response_deserialization() {
        let json = json!({
            "values": [
                {
                    "slug": "repo1",
                    "name": "Repository 1",
                    "public": true,
                    "project": {
                        "key": "PROJ",
                        "name": "Project",
                        "type": "NORMAL"
                    }
                },
                {
                    "slug": "repo2",
                    "name": "Repository 2",
                    "public": false,
                    "project": {
                        "key": "PROJ",
                        "name": "Project",
                        "type": "NORMAL"
                    }
                }
            ],
            "nextPageStart": 25,
            "isLastPage": false
        });
        
        let response: PaginatedResponse<Repository> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].slug, "repo1");
        assert_eq!(response.values[1].slug, "repo2");
        assert_eq!(response.next_page_start, Some(25));
        assert_eq!(response.is_last_page, false);
    }
    
    #[test]
    fn test_clone_url_helpers() {
        let json = json!({
            "slug": "my-repo",
            "name": "My Repository",
            "project": {
                "key": "PROJ",
                "name": "Project",
                "type": "NORMAL"
            },
            "links": {
                "clone": [
                    {
                        "href": "https://example.com/repo.git",
                        "name": "http"
                    }
                ]
            }
        });
        
        let repo: Repository = serde_json::from_value(json).unwrap();
        assert_eq!(repo.http_clone_url(), Some("https://example.com/repo.git".to_string()));
        assert_eq!(repo.ssh_clone_url(), None);
    }
}
