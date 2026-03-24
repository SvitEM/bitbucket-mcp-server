use serde::{Deserialize, Serialize};
use crate::api::error::ApiError;
use crate::api::BitbucketClient;
use crate::types::PaginatedResponse;

/// Represents a Bitbucket Server project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// Unique project key (e.g., "PROJ")
    pub key: String,
    /// Human-readable project name
    pub name: String,
    /// Optional project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the project is publicly accessible
    #[serde(default)]
    pub public: bool,
    /// Project type (typically "NORMAL")
    #[serde(rename = "type")]
    pub project_type: String,
}

impl BitbucketClient {
    /// Lists all projects with pagination
    /// 
    /// # Arguments
    /// * `start` - Starting index for pagination (0-based)
    /// * `limit` - Maximum number of projects to return
    /// 
    /// # Returns
    /// A paginated response containing projects
    pub async fn list_projects(&self, start: u32, limit: u32) -> Result<PaginatedResponse<Project>, ApiError> {
        let url = self.build_paginated_url("projects", start, limit);
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<Project> = response.json().await?;
        Ok(json)
    }
    
    /// Gets a specific project by its key
    /// 
    /// # Arguments
    /// * `project_key` - The unique project key (e.g., "PROJ")
    /// 
    /// # Returns
    /// The requested project
    pub async fn get_project(&self, project_key: &str) -> Result<Project, ApiError> {
        let url = self.build_url(&format!("projects/{}", project_key));
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let project: Project = response.json().await?;
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_project_deserialization() {
        let json = json!({
            "key": "PROJ",
            "name": "My Project",
            "description": "A test project",
            "public": false,
            "type": "NORMAL"
        });
        
        let project: Project = serde_json::from_value(json).unwrap();
        assert_eq!(project.key, "PROJ");
        assert_eq!(project.name, "My Project");
        assert_eq!(project.description, Some("A test project".to_string()));
        assert_eq!(project.public, false);
        assert_eq!(project.project_type, "NORMAL");
    }
    
    #[test]
    fn test_project_deserialization_minimal() {
        let json = json!({
            "key": "TEST",
            "name": "Test Project",
            "type": "NORMAL"
        });
        
        let project: Project = serde_json::from_value(json).unwrap();
        assert_eq!(project.key, "TEST");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, None);
        assert_eq!(project.public, false);
        assert_eq!(project.project_type, "NORMAL");
    }
    
    #[test]
    fn test_paginated_response_deserialization() {
        let json = json!({
            "values": [
                {
                    "key": "PROJ1",
                    "name": "Project 1",
                    "type": "NORMAL",
                    "public": true
                },
                {
                    "key": "PROJ2",
                    "name": "Project 2",
                    "type": "NORMAL",
                    "public": false
                }
            ],
            "nextPageStart": 25,
            "isLastPage": false
        });
        
        let response: PaginatedResponse<Project> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].key, "PROJ1");
        assert_eq!(response.values[1].key, "PROJ2");
        assert_eq!(response.next_page_start, Some(25));
        assert_eq!(response.is_last_page, false);
    }
}
