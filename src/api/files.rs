use serde::{Deserialize, Serialize};
use crate::api::error::ApiError;
use crate::api::BitbucketClient;
use crate::types::PaginatedResponse;

/// Represents a file path in Bitbucket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilePath {
    /// Path components (e.g., ["src", "main.rs"])
    pub components: Vec<String>,
    /// Parent directory path
    pub parent: String,
    /// File or directory name
    pub name: String,
    /// File extension (optional, only for files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    /// Full path as string
    #[serde(rename = "toString")]
    pub to_string: String,
}

/// Represents a file or directory entry in a repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    /// Path information
    pub path: FilePath,
    /// Entry type: "FILE" or "DIRECTORY"
    #[serde(rename = "type")]
    pub entry_type: String,
    /// File size in bytes (only for files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Content ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentId")]
    pub content_id: Option<String>,
}

impl BitbucketClient {
    /// Lists files and directories at a given path in a repository
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `path` - The path to browse (empty string for root)
    /// * `at` - The commit hash or branch name to browse at (e.g., "main", "refs/heads/main")
    /// * `start` - Starting index for pagination (0-based)
    /// * `limit` - Maximum number of entries to return
    /// 
    /// # Returns
    /// A paginated response containing file entries
    pub async fn list_files(
        &self,
        project_key: &str,
        repo_slug: &str,
        path: &str,
        at: Option<&str>,
        start: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<FileEntry>, ApiError> {
        let base_path = if path.is_empty() {
            format!("projects/{}/repos/{}/browse", project_key, repo_slug)
        } else {
            format!("projects/{}/repos/{}/browse/{}", project_key, repo_slug, path)
        };
        
        let mut url = self.build_paginated_url(&base_path, start, limit);
        
        if let Some(at_ref) = at {
            url = format!("{}&at={}", url, at_ref);
        }
        
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let json: PaginatedResponse<FileEntry> = response.json().await?;
        Ok(json)
    }
    
    /// Gets the raw content of a file
    /// 
    /// # Arguments
    /// * `project_key` - The project key (e.g., "PROJ")
    /// * `repo_slug` - The repository slug (e.g., "my-repo")
    /// * `path` - The file path (e.g., "src/main.rs")
    /// * `at` - The commit hash or branch name (e.g., "main", "refs/heads/main")
    /// 
    /// # Returns
    /// The raw file content as a string
    pub async fn get_file_content(
        &self,
        project_key: &str,
        repo_slug: &str,
        path: &str,
        at: Option<&str>,
    ) -> Result<String, ApiError> {
        let mut url = self.build_url(&format!(
            "projects/{}/repos/{}/raw/{}",
            project_key, repo_slug, path
        ));
        
        if let Some(at_ref) = at {
            url = format!("{}?at={}", url, at_ref);
        }
        
        let headers = self.default_headers()?;
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;
        
        let content: String = response.text().await?;
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_file_entry_deserialization() {
        let json = json!({
            "path": {
                "components": ["src", "main.rs"],
                "parent": "src",
                "name": "main.rs",
                "extension": "rs",
                "toString": "src/main.rs"
            },
            "type": "FILE",
            "size": 1024,
            "contentId": "abc123def456"
        });
        
        let entry: FileEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.path.name, "main.rs");
        assert_eq!(entry.path.parent, "src");
        assert_eq!(entry.path.extension, Some("rs".to_string()));
        assert_eq!(entry.path.to_string, "src/main.rs");
        assert_eq!(entry.entry_type, "FILE");
        assert_eq!(entry.size, Some(1024));
        assert_eq!(entry.content_id, Some("abc123def456".to_string()));
    }
    
    #[test]
    fn test_directory_entry_deserialization() {
        let json = json!({
            "path": {
                "components": ["src"],
                "parent": "",
                "name": "src",
                "toString": "src"
            },
            "type": "DIRECTORY"
        });
        
        let entry: FileEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.path.name, "src");
        assert_eq!(entry.path.parent, "");
        assert_eq!(entry.path.extension, None);
        assert_eq!(entry.entry_type, "DIRECTORY");
        assert_eq!(entry.size, None);
        assert_eq!(entry.content_id, None);
    }
    
    #[test]
    fn test_paginated_files_deserialization() {
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
                    "type": "FILE",
                    "size": 512
                },
                {
                    "path": {
                        "components": ["src"],
                        "parent": "",
                        "name": "src",
                        "toString": "src"
                    },
                    "type": "DIRECTORY"
                }
            ],
            "isLastPage": true
        });
        
        let response: PaginatedResponse<FileEntry> = serde_json::from_value(json).unwrap();
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].entry_type, "FILE");
        assert_eq!(response.values[0].path.name, "README.md");
        assert_eq!(response.values[1].entry_type, "DIRECTORY");
        assert_eq!(response.values[1].path.name, "src");
        assert_eq!(response.is_last_page, true);
    }
}
