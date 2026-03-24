use serde::{Deserialize, Serialize};
use crate::api::error::ApiError;
use crate::api::BitbucketClient;
use crate::types::PaginatedResponse;

/// Represents commit author or committer information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitAuthor {
    /// Author/committer name
    pub name: String,
    /// Author/committer email address
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    /// Display name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

/// Represents a parent commit reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitParent {
    /// Parent commit hash
    pub id: String,
    /// Short parent commit hash for display
    #[serde(rename = "displayId")]
    pub display_id: String,
}

/// Represents a commit in a Bitbucket repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commit {
    /// Full commit hash
    pub id: String,
    /// Short commit hash for display
    #[serde(rename = "displayId")]
    pub display_id: String,
    /// Commit author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<CommitAuthor>,
    /// Author timestamp in milliseconds since epoch
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "authorTimestamp")]
    pub author_timestamp: Option<i64>,
    /// Committer information (optional, may differ from author)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committer: Option<CommitAuthor>,
    /// Committer timestamp in milliseconds since epoch
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "committerTimestamp")]
    pub committer_timestamp: Option<i64>,
    /// Commit message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Parent commits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<CommitParent>>,
}

impl BitbucketClient {
    /// Lists commits in a repository with pagination
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `start` - Starting index for pagination (0-based)
    /// * `limit` - Maximum number of commits to return
    /// 
    /// # Returns
    /// A paginated response containing commits
    pub async fn list_commits(
        &self,
        project_key: &str,
        repo_slug: &str,
        start: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<Commit>, ApiError> {
        let path = format!("projects/{}/repos/{}/commits", project_key, repo_slug);
        let url = self.build_paginated_url(&path, start, limit);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<Commit> = response.json().await?;
        Ok(json)
    }
    
    /// Gets a specific commit by its hash
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `commit_id` - The commit hash (full or short)
    /// 
    /// # Returns
    /// The requested commit
    pub async fn get_commit(
        &self,
        project_key: &str,
        repo_slug: &str,
        commit_id: &str,
    ) -> Result<Commit, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/commits/{}",
            project_key, repo_slug, commit_id
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let commit: Commit = response.json().await?;
        Ok(commit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_commit_deserialization_full() {
        let json = json!({
            "id": "abcdef1234567890abcdef1234567890abcdef12",
            "displayId": "abcdef1",
            "author": {
                "name": "John Doe",
                "emailAddress": "john@example.com",
                "displayName": "John Doe"
            },
            "authorTimestamp": 1609459200000i64,
            "committer": {
                "name": "Jane Smith",
                "emailAddress": "jane@example.com",
                "displayName": "Jane Smith"
            },
            "committerTimestamp": 1609459300000i64,
            "message": "Initial commit\n\nAdded project structure",
            "parents": [
                {
                    "id": "1234567890abcdef1234567890abcdef12345678",
                    "displayId": "1234567"
                }
            ]
        });
        
        let commit: Commit = serde_json::from_value(json).unwrap();
        assert_eq!(commit.id, "abcdef1234567890abcdef1234567890abcdef12");
        assert_eq!(commit.display_id, "abcdef1");
        
        let author = commit.author.unwrap();
        assert_eq!(author.name, "John Doe");
        assert_eq!(author.email_address, "john@example.com");
        assert_eq!(author.display_name, Some("John Doe".to_string()));
        
        assert_eq!(commit.author_timestamp, Some(1609459200000));
        
        let committer = commit.committer.unwrap();
        assert_eq!(committer.name, "Jane Smith");
        assert_eq!(committer.email_address, "jane@example.com");
        
        assert_eq!(commit.committer_timestamp, Some(1609459300000));
        assert_eq!(commit.message, Some("Initial commit\n\nAdded project structure".to_string()));
        
        let parents = commit.parents.unwrap();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].id, "1234567890abcdef1234567890abcdef12345678");
        assert_eq!(parents[0].display_id, "1234567");
    }
    
    #[test]
    fn test_commit_deserialization_minimal() {
        let json = json!({
            "id": "abcdef1234567890",
            "displayId": "abcdef1"
        });
        
        let commit: Commit = serde_json::from_value(json).unwrap();
        assert_eq!(commit.id, "abcdef1234567890");
        assert_eq!(commit.display_id, "abcdef1");
        assert!(commit.author.is_none());
        assert!(commit.author_timestamp.is_none());
        assert!(commit.committer.is_none());
        assert!(commit.committer_timestamp.is_none());
        assert!(commit.message.is_none());
        assert!(commit.parents.is_none());
    }
    
    #[test]
    fn test_commit_author_deserialization() {
        let json = json!({
            "name": "Alice Developer",
            "emailAddress": "alice@example.com",
            "displayName": "Alice D."
        });
        
        let author: CommitAuthor = serde_json::from_value(json).unwrap();
        assert_eq!(author.name, "Alice Developer");
        assert_eq!(author.email_address, "alice@example.com");
        assert_eq!(author.display_name, Some("Alice D.".to_string()));
    }
    
    #[test]
    fn test_commit_author_deserialization_minimal() {
        let json = json!({
            "name": "Bob Builder",
            "emailAddress": "bob@example.com"
        });
        
        let author: CommitAuthor = serde_json::from_value(json).unwrap();
        assert_eq!(author.name, "Bob Builder");
        assert_eq!(author.email_address, "bob@example.com");
        assert!(author.display_name.is_none());
    }
    
    #[test]
    fn test_commit_parent_deserialization() {
        let json = json!({
            "id": "fedcba0987654321fedcba0987654321fedcba09",
            "displayId": "fedcba0"
        });
        
        let parent: CommitParent = serde_json::from_value(json).unwrap();
        assert_eq!(parent.id, "fedcba0987654321fedcba0987654321fedcba09");
        assert_eq!(parent.display_id, "fedcba0");
    }
    
    #[test]
    fn test_paginated_commits_deserialization() {
        let json = json!({
            "values": [
                {
                    "id": "commit1",
                    "displayId": "commit1",
                    "message": "First commit"
                },
                {
                    "id": "commit2",
                    "displayId": "commit2",
                    "message": "Second commit",
                    "author": {
                        "name": "Dev User",
                        "emailAddress": "dev@example.com"
                    }
                }
            ],
            "nextPageStart": 50,
            "isLastPage": false
        });
        
        let response: PaginatedResponse<Commit> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].id, "commit1");
        assert_eq!(response.values[0].message, Some("First commit".to_string()));
        assert_eq!(response.values[1].id, "commit2");
        assert!(response.values[1].author.is_some());
        assert_eq!(response.next_page_start, Some(50));
        assert_eq!(response.is_last_page, false);
    }
}
