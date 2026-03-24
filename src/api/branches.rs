use serde::{Deserialize, Serialize};
use crate::api::error::ApiError;
use crate::api::BitbucketClient;
use crate::types::PaginatedResponse;

/// Represents a branch in a Bitbucket repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Branch {
    /// Full reference name (e.g., "refs/heads/main")
    pub id: String,
    /// Display name (e.g., "main")
    #[serde(rename = "displayId")]
    pub display_id: String,
    /// Branch type (typically "BRANCH")
    #[serde(rename = "type")]
    pub branch_type: String,
    /// Latest commit hash
    #[serde(rename = "latestCommit")]
    pub latest_commit: String,
}

/// Represents a commit in branch comparison results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commit {
    /// Commit hash
    pub id: String,
    /// Short commit hash for display
    #[serde(rename = "displayId")]
    pub display_id: String,
    /// Commit author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<CommitAuthor>,
    /// Commit message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Represents commit author information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitAuthor {
    /// Author name
    pub name: String,
    /// Author email address
    #[serde(rename = "emailAddress")]
    pub email_address: String,
}

/// Request body for creating a new branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchRequest {
    /// Name of the new branch
    pub name: String,
    /// Starting point (commit hash or branch name)
    #[serde(rename = "startPoint")]
    pub start_point: String,
}

impl BitbucketClient {
    /// Lists branches in a repository with pagination
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `start` - Starting index for pagination (0-based)
    /// * `limit` - Maximum number of branches to return
    /// 
    /// # Returns
    /// A paginated response containing branches
    pub async fn list_branches(
        &self,
        project_key: &str,
        repo_slug: &str,
        start: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<Branch>, ApiError> {
        let path = format!("projects/{}/repos/{}/branches", project_key, repo_slug);
        let url = self.build_paginated_url(&path, start, limit);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<Branch> = response.json().await?;
        Ok(json)
    }
    
    /// Creates a new branch in a repository
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `name` - Name of the new branch
    /// * `start_point` - Starting point (commit hash or branch name)
    /// 
    /// # Returns
    /// The created branch
    pub async fn create_branch(
        &self,
        project_key: &str,
        repo_slug: &str,
        name: &str,
        start_point: &str,
    ) -> Result<Branch, ApiError> {
        let url = self.build_url(&format!("projects/{}/repos/{}/branches", project_key, repo_slug));
        let headers = self.default_headers()?;
        
        let request_body = CreateBranchRequest {
            name: name.to_string(),
            start_point: start_point.to_string(),
        };
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;
        
        let branch: Branch = response.json().await?;
        Ok(branch)
    }
    
    /// Compares commits between two branches
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `from` - Source branch or commit
    /// * `to` - Target branch or commit
    /// 
    /// # Returns
    /// A paginated response containing commits that differ
    pub async fn branch_compare(
        &self,
        project_key: &str,
        repo_slug: &str,
        from: &str,
        to: &str,
    ) -> Result<PaginatedResponse<Commit>, ApiError> {
        let path = format!("projects/{}/repos/{}/compare/commits", project_key, repo_slug);
        let base_url = self.build_url(&path);
        let url = format!("{}?from={}&to={}", base_url, from, to);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<Commit> = response.json().await?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_branch_deserialization() {
        let json = json!({
            "id": "refs/heads/main",
            "displayId": "main",
            "type": "BRANCH",
            "latestCommit": "abcdef1234567890"
        });
        
        let branch: Branch = serde_json::from_value(json).unwrap();
        assert_eq!(branch.id, "refs/heads/main");
        assert_eq!(branch.display_id, "main");
        assert_eq!(branch.branch_type, "BRANCH");
        assert_eq!(branch.latest_commit, "abcdef1234567890");
    }
    
    #[test]
    fn test_commit_deserialization() {
        let json = json!({
            "id": "abcdef1234567890",
            "displayId": "abcdef1",
            "author": {
                "name": "John Doe",
                "emailAddress": "john@example.com"
            },
            "message": "Initial commit"
        });
        
        let commit: Commit = serde_json::from_value(json).unwrap();
        assert_eq!(commit.id, "abcdef1234567890");
        assert_eq!(commit.display_id, "abcdef1");
        assert!(commit.author.is_some());
        assert_eq!(commit.author.as_ref().unwrap().name, "John Doe");
        assert_eq!(commit.author.as_ref().unwrap().email_address, "john@example.com");
        assert_eq!(commit.message, Some("Initial commit".to_string()));
    }
    
    #[test]
    fn test_commit_deserialization_minimal() {
        let json = json!({
            "id": "1234567890abcdef",
            "displayId": "1234567"
        });
        
        let commit: Commit = serde_json::from_value(json).unwrap();
        assert_eq!(commit.id, "1234567890abcdef");
        assert_eq!(commit.display_id, "1234567");
        assert!(commit.author.is_none());
        assert!(commit.message.is_none());
    }
    
    #[test]
    fn test_paginated_branches_deserialization() {
        let json = json!({
            "values": [
                {
                    "id": "refs/heads/main",
                    "displayId": "main",
                    "type": "BRANCH",
                    "latestCommit": "abc123"
                },
                {
                    "id": "refs/heads/develop",
                    "displayId": "develop",
                    "type": "BRANCH",
                    "latestCommit": "def456"
                }
            ],
            "nextPageStart": 25,
            "isLastPage": false
        });
        
        let response: PaginatedResponse<Branch> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].display_id, "main");
        assert_eq!(response.values[1].display_id, "develop");
        assert_eq!(response.next_page_start, Some(25));
        assert_eq!(response.is_last_page, false);
    }
    
    #[test]
    fn test_create_branch_request_serialization() {
        let request = CreateBranchRequest {
            name: "feature/new-branch".to_string(),
            start_point: "main".to_string(),
        };
        
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "feature/new-branch");
        assert_eq!(json["startPoint"], "main");
    }
    
    #[test]
    fn test_paginated_commits_deserialization() {
        let json = json!({
            "values": [
                {
                    "id": "abc123",
                    "displayId": "abc123",
                    "message": "First commit"
                },
                {
                    "id": "def456",
                    "displayId": "def456",
                    "message": "Second commit"
                }
            ],
            "isLastPage": true
        });
        
        let response: PaginatedResponse<Commit> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].id, "abc123");
        assert_eq!(response.values[1].id, "def456");
        assert_eq!(response.is_last_page, true);
    }
}
