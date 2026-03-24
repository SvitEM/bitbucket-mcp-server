use serde::{Deserialize, Serialize};
use crate::api::error::ApiError;
use crate::api::BitbucketClient;
use crate::types::PaginatedResponse;

/// Represents a user in Bitbucket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// User name
    pub name: String,
    /// Email address
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    /// Display name
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// User ID
    pub id: u64,
    /// User slug
    pub slug: String,
}

/// Represents a repository reference in a pull request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    /// Repository slug
    pub slug: String,
    /// Repository name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Project key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectRef>,
}

/// Represents a project reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectRef {
    /// Project key
    pub key: String,
}

/// Represents a pull request reference (fromRef or toRef)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequestRef {
    /// Branch or tag ID
    pub id: String,
    /// Repository information
    pub repository: Repository,
    /// Display ID (branch name)
    #[serde(rename = "displayId")]
    pub display_id: String,
    /// Latest commit hash
    #[serde(rename = "latestCommit", skip_serializing_if = "Option::is_none")]
    pub latest_commit: Option<String>,
}

/// Represents a pull request participant (reviewer)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequestParticipant {
    /// User information
    pub user: User,
    /// Participant role (REVIEWER, AUTHOR, PARTICIPANT)
    pub role: String,
    /// Whether the participant approved
    pub approved: bool,
    /// Participant status (APPROVED, UNAPPROVED, NEEDS_WORK)
    pub status: String,
}

/// Represents a pull request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequest {
    /// Pull request ID
    pub id: u64,
    /// Version number for optimistic locking
    pub version: u64,
    /// Pull request title
    pub title: String,
    /// Pull request description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Pull request state (OPEN, MERGED, DECLINED)
    pub state: String,
    /// Whether the PR is open
    pub open: bool,
    /// Whether the PR is closed
    pub closed: bool,
    /// Creation timestamp
    #[serde(rename = "createdDate")]
    pub created_date: u64,
    /// Last update timestamp
    #[serde(rename = "updatedDate")]
    pub updated_date: u64,
    /// Source branch reference
    #[serde(rename = "fromRef")]
    pub from_ref: PullRequestRef,
    /// Target branch reference
    #[serde(rename = "toRef")]
    pub to_ref: PullRequestRef,
    /// Pull request author
    pub author: PullRequestParticipant,
    /// List of reviewers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<PullRequestParticipant>>,
}

/// Request body for creating a pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestRequest {
    /// Pull request title
    pub title: String,
    /// Pull request description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Source branch reference
    #[serde(rename = "fromRef")]
    pub from_ref: RefRequest,
    /// Target branch reference
    #[serde(rename = "toRef")]
    pub to_ref: RefRequest,
    /// List of reviewer user names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<ReviewerRequest>>,
}

/// Reference request for creating/updating PRs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefRequest {
    /// Branch ID (e.g., "refs/heads/feature-branch")
    pub id: String,
    /// Repository information
    pub repository: RepositoryRequest,
}

/// Repository request for creating/updating PRs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRequest {
    /// Repository slug
    pub slug: String,
    /// Project information
    pub project: ProjectRequest,
}

/// Project request for creating/updating PRs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRequest {
    /// Project key
    pub key: String,
}

/// Reviewer request for creating/updating PRs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerRequest {
    /// User information
    pub user: UserRequest,
}

/// User request for creating/updating PRs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequest {
    /// User name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestRequest {
    pub version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<ReviewerRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    pub path: ChangePath,
    #[serde(rename = "type")]
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangePath {
    pub components: Vec<String>,
    pub parent: String,
    pub name: String,
    pub extension: Option<String>,
    #[serde(rename = "toString")]
    pub to_string: String,
}

impl BitbucketClient {
    pub async fn list_prs(
        &self,
        project_key: &str,
        repo_slug: &str,
        start: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<PullRequest>, ApiError> {
        let path = format!("projects/{}/repos/{}/pull-requests", project_key, repo_slug);
        let url = self.build_paginated_url(&path, start, limit);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<PullRequest> = response.json().await?;
        Ok(json)
    }
    
    pub async fn get_pr(
        &self,
        project_key: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/pull-requests/{}",
            project_key, repo_slug, pr_id
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let pr: PullRequest = response.json().await?;
        Ok(pr)
    }
    
    pub async fn create_pr(
        &self,
        project_key: &str,
        repo_slug: &str,
        request: CreatePullRequestRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/pull-requests",
            project_key, repo_slug
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
        
        let pr: PullRequest = response.json().await?;
        Ok(pr)
    }
    
    pub async fn update_pr(
        &self,
        project_key: &str,
        repo_slug: &str,
        pr_id: u64,
        request: UpdatePullRequestRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/pull-requests/{}",
            project_key, repo_slug, pr_id
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .put(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
        
        let pr: PullRequest = response.json().await?;
        Ok(pr)
    }
    
    pub async fn merge_pr(
        &self,
        project_key: &str,
        repo_slug: &str,
        pr_id: u64,
        version: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/pull-requests/{}/merge?version={}",
            project_key, repo_slug, pr_id, version
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .send()
            .await?;
        
        let pr: PullRequest = response.json().await?;
        Ok(pr)
    }
    
    pub async fn decline_pr(
        &self,
        project_key: &str,
        repo_slug: &str,
        pr_id: u64,
        version: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/pull-requests/{}/decline?version={}",
            project_key, repo_slug, pr_id, version
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .send()
            .await?;
        
        let pr: PullRequest = response.json().await?;
        Ok(pr)
    }
    
    pub async fn pr_diff(
        &self,
        project_key: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<String, ApiError> {
        let url = self.build_url(&format!(
            "projects/{}/repos/{}/pull-requests/{}/diff",
            project_key, repo_slug, pr_id
        ));
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let diff: String = response.text().await?;
        Ok(diff)
    }
    
    pub async fn pr_changes(
        &self,
        project_key: &str,
        repo_slug: &str,
        pr_id: u64,
        start: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<Change>, ApiError> {
        let path = format!(
            "projects/{}/repos/{}/pull-requests/{}/changes",
            project_key, repo_slug, pr_id
        );
        let url = self.build_paginated_url(&path, start, limit);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<Change> = response.json().await?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_pull_request_deserialization() {
        let json = json!({
            "id": 101,
            "version": 1,
            "title": "Feature: Add new API",
            "description": "This PR adds a new API endpoint",
            "state": "OPEN",
            "open": true,
            "closed": false,
            "createdDate": 1609459200000u64,
            "updatedDate": 1609545600000u64,
            "fromRef": {
                "id": "refs/heads/feature-branch",
                "displayId": "feature-branch",
                "latestCommit": "abc123",
                "repository": {
                    "slug": "my-repo",
                    "name": "My Repository",
                    "project": {
                        "key": "PROJ"
                    }
                }
            },
            "toRef": {
                "id": "refs/heads/main",
                "displayId": "main",
                "latestCommit": "def456",
                "repository": {
                    "slug": "my-repo",
                    "project": {
                        "key": "PROJ"
                    }
                }
            },
            "author": {
                "user": {
                    "name": "jdoe",
                    "emailAddress": "jdoe@example.com",
                    "displayName": "John Doe",
                    "id": 1,
                    "slug": "jdoe"
                },
                "role": "AUTHOR",
                "approved": false,
                "status": "UNAPPROVED"
            },
            "reviewers": [
                {
                    "user": {
                        "name": "reviewer1",
                        "emailAddress": "reviewer1@example.com",
                        "displayName": "Reviewer One",
                        "id": 2,
                        "slug": "reviewer1"
                    },
                    "role": "REVIEWER",
                    "approved": true,
                    "status": "APPROVED"
                }
            ]
        });
        
        let pr: PullRequest = serde_json::from_value(json).unwrap();
        assert_eq!(pr.id, 101);
        assert_eq!(pr.version, 1);
        assert_eq!(pr.title, "Feature: Add new API");
        assert_eq!(pr.description, Some("This PR adds a new API endpoint".to_string()));
        assert_eq!(pr.state, "OPEN");
        assert_eq!(pr.open, true);
        assert_eq!(pr.closed, false);
        assert_eq!(pr.from_ref.display_id, "feature-branch");
        assert_eq!(pr.to_ref.display_id, "main");
        assert_eq!(pr.author.user.name, "jdoe");
        assert!(pr.reviewers.is_some());
        assert_eq!(pr.reviewers.as_ref().unwrap().len(), 1);
    }
    
    #[test]
    fn test_pull_request_deserialization_minimal() {
        let json = json!({
            "id": 102,
            "version": 2,
            "title": "Quick fix",
            "state": "MERGED",
            "open": false,
            "closed": true,
            "createdDate": 1609459200000u64,
            "updatedDate": 1609545600000u64,
            "fromRef": {
                "id": "refs/heads/hotfix",
                "displayId": "hotfix",
                "repository": {
                    "slug": "repo"
                }
            },
            "toRef": {
                "id": "refs/heads/main",
                "displayId": "main",
                "repository": {
                    "slug": "repo"
                }
            },
            "author": {
                "user": {
                    "name": "admin",
                    "emailAddress": "admin@example.com",
                    "displayName": "Admin",
                    "id": 99,
                    "slug": "admin"
                },
                "role": "AUTHOR",
                "approved": false,
                "status": "UNAPPROVED"
            }
        });
        
        let pr: PullRequest = serde_json::from_value(json).unwrap();
        assert_eq!(pr.id, 102);
        assert_eq!(pr.title, "Quick fix");
        assert_eq!(pr.description, None);
        assert_eq!(pr.state, "MERGED");
        assert_eq!(pr.reviewers, None);
    }
    
    #[test]
    fn test_participant_deserialization() {
        let json = json!({
            "user": {
                "name": "testuser",
                "emailAddress": "test@example.com",
                "displayName": "Test User",
                "id": 42,
                "slug": "testuser"
            },
            "role": "REVIEWER",
            "approved": true,
            "status": "APPROVED"
        });
        
        let participant: PullRequestParticipant = serde_json::from_value(json).unwrap();
        assert_eq!(participant.user.name, "testuser");
        assert_eq!(participant.role, "REVIEWER");
        assert_eq!(participant.approved, true);
        assert_eq!(participant.status, "APPROVED");
    }
    
    #[test]
    fn test_paginated_prs_deserialization() {
        let json = json!({
            "values": [
                {
                    "id": 1,
                    "version": 1,
                    "title": "PR 1",
                    "state": "OPEN",
                    "open": true,
                    "closed": false,
                    "createdDate": 1609459200000u64,
                    "updatedDate": 1609545600000u64,
                    "fromRef": {
                        "id": "refs/heads/feature1",
                        "displayId": "feature1",
                        "repository": {"slug": "repo"}
                    },
                    "toRef": {
                        "id": "refs/heads/main",
                        "displayId": "main",
                        "repository": {"slug": "repo"}
                    },
                    "author": {
                        "user": {
                            "name": "user1",
                            "emailAddress": "user1@example.com",
                            "displayName": "User One",
                            "id": 1,
                            "slug": "user1"
                        },
                        "role": "AUTHOR",
                        "approved": false,
                        "status": "UNAPPROVED"
                    }
                },
                {
                    "id": 2,
                    "version": 1,
                    "title": "PR 2",
                    "state": "OPEN",
                    "open": true,
                    "closed": false,
                    "createdDate": 1609459200000u64,
                    "updatedDate": 1609545600000u64,
                    "fromRef": {
                        "id": "refs/heads/feature2",
                        "displayId": "feature2",
                        "repository": {"slug": "repo"}
                    },
                    "toRef": {
                        "id": "refs/heads/main",
                        "displayId": "main",
                        "repository": {"slug": "repo"}
                    },
                    "author": {
                        "user": {
                            "name": "user2",
                            "emailAddress": "user2@example.com",
                            "displayName": "User Two",
                            "id": 2,
                            "slug": "user2"
                        },
                        "role": "AUTHOR",
                        "approved": false,
                        "status": "UNAPPROVED"
                    }
                }
            ],
            "nextPageStart": 25,
            "isLastPage": false
        });
        
        let response: PaginatedResponse<PullRequest> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].title, "PR 1");
        assert_eq!(response.values[1].title, "PR 2");
        assert_eq!(response.next_page_start, Some(25));
        assert_eq!(response.is_last_page, false);
    }
    
    #[test]
    fn test_create_pr_request_serialization() {
        let request = CreatePullRequestRequest {
            title: "New Feature".to_string(),
            description: Some("Feature description".to_string()),
            from_ref: RefRequest {
                id: "refs/heads/feature".to_string(),
                repository: RepositoryRequest {
                    slug: "my-repo".to_string(),
                    project: ProjectRequest {
                        key: "PROJ".to_string(),
                    },
                },
            },
            to_ref: RefRequest {
                id: "refs/heads/main".to_string(),
                repository: RepositoryRequest {
                    slug: "my-repo".to_string(),
                    project: ProjectRequest {
                        key: "PROJ".to_string(),
                    },
                },
            },
            reviewers: Some(vec![
                ReviewerRequest {
                    user: UserRequest {
                        name: "reviewer1".to_string(),
                    },
                },
            ]),
        };
        
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["title"], "New Feature");
        assert_eq!(json["description"], "Feature description");
        assert_eq!(json["fromRef"]["id"], "refs/heads/feature");
        assert_eq!(json["toRef"]["id"], "refs/heads/main");
        assert_eq!(json["reviewers"][0]["user"]["name"], "reviewer1");
    }
    
    #[test]
    fn test_update_pr_request_serialization() {
        let request = UpdatePullRequestRequest {
            version: 5,
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            reviewers: None,
        };
        
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["version"], 5);
        assert_eq!(json["title"], "Updated Title");
        assert_eq!(json["description"], "Updated description");
        assert!(json.get("reviewers").is_none());
    }
    
    #[test]
    fn test_change_deserialization() {
        let json = json!({
            "path": {
                "components": ["src", "main.rs"],
                "parent": "src",
                "name": "main.rs",
                "extension": "rs",
                "toString": "src/main.rs"
            },
            "type": "MODIFY"
        });
        
        let change: Change = serde_json::from_value(json).unwrap();
        assert_eq!(change.path.name, "main.rs");
        assert_eq!(change.path.parent, "src");
        assert_eq!(change.path.extension, Some("rs".to_string()));
        assert_eq!(change.change_type, "MODIFY");
    }
    
    #[test]
    fn test_paginated_changes_deserialization() {
        let json = json!({
            "values": [
                {
                    "path": {
                        "components": ["README.md"],
                        "parent": "",
                        "name": "README.md",
                        "extension": "md",
                        "toString": "README.md"
                    },
                    "type": "ADD"
                },
                {
                    "path": {
                        "components": ["src", "lib.rs"],
                        "parent": "src",
                        "name": "lib.rs",
                        "extension": "rs",
                        "toString": "src/lib.rs"
                    },
                    "type": "MODIFY"
                }
            ],
            "isLastPage": true
        });
        
        let response: PaginatedResponse<Change> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].change_type, "ADD");
        assert_eq!(response.values[1].change_type, "MODIFY");
        assert_eq!(response.is_last_page, true);
    }
}
