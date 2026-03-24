use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::*,
    service::RequestContext,
};
use serde_json::json;
use std::sync::Arc;
use crate::api::BitbucketClient;
use crate::api::pull_requests::{CreatePullRequestRequest, UpdatePullRequestRequest, RefRequest, RepositoryRequest, ProjectRequest, ReviewerRequest, UserRequest};
use crate::config;
use crate::permission::{PermissionChecker, Operation};

#[derive(Clone, Default)]
pub struct BitbucketMcpServer;

impl BitbucketMcpServer {
    pub fn new() -> Self {
        Self
    }
}

fn schema(v: serde_json::Value) -> Arc<serde_json::Map<String, serde_json::Value>> {
    Arc::new(v.as_object().expect("Schema must be a JSON object").clone())
}

impl ServerHandler for BitbucketMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = vec![
            Tool::new(
                "bitbucket_list_projects",
                "Lists Bitbucket projects with pagination",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    }
                }))
            ),
            Tool::new(
                "bitbucket_get_project",
                "Gets a specific project by key",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"}
                    },
                    "required": ["project_key"]
                }))
            ),
            Tool::new(
                "bitbucket_list_repos",
                "Lists repositories in a project",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    },
                    "required": ["project_key"]
                }))
            ),
            Tool::new(
                "bitbucket_get_repo",
                "Gets a specific repository",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"}
                    },
                    "required": ["project_key", "repo_slug"]
                }))
            ),
            Tool::new(
                "bitbucket_list_prs",
                "Lists pull requests",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    },
                    "required": ["project_key", "repo_slug"]
                }))
            ),
            Tool::new(
                "bitbucket_get_pr",
                "Gets a specific pull request",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "pr_id": {"type": "number"}
                    },
                    "required": ["project_key", "repo_slug", "pr_id"]
                }))
            ),
            Tool::new(
                "bitbucket_create_pr",
                "Creates a new pull request",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "title": {"type": "string"},
                        "description": {"type": "string"},
                        "from_branch": {"type": "string"},
                        "to_branch": {"type": "string"},
                        "reviewers": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["project_key", "repo_slug", "title", "from_branch", "to_branch"]
                }))
            ),
            Tool::new(
                "bitbucket_update_pr",
                "Updates a pull request",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "pr_id": {"type": "number"},
                        "version": {"type": "number"},
                        "title": {"type": "string"},
                        "description": {"type": "string"}
                    },
                    "required": ["project_key", "repo_slug", "pr_id", "version"]
                }))
            ),
            Tool::new(
                "bitbucket_merge_pr",
                "Merges a pull request",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "pr_id": {"type": "number"},
                        "version": {"type": "number"}
                    },
                    "required": ["project_key", "repo_slug", "pr_id", "version"]
                }))
            ),
            Tool::new(
                "bitbucket_decline_pr",
                "Declines a pull request",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "pr_id": {"type": "number"},
                        "version": {"type": "number"}
                    },
                    "required": ["project_key", "repo_slug", "pr_id", "version"]
                }))
            ),
            Tool::new(
                "bitbucket_pr_diff",
                "Gets PR diff",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "pr_id": {"type": "number"}
                    },
                    "required": ["project_key", "repo_slug", "pr_id"]
                }))
            ),
            Tool::new(
                "bitbucket_pr_changes",
                "Lists PR file changes",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "pr_id": {"type": "number"},
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    },
                    "required": ["project_key", "repo_slug", "pr_id"]
                }))
            ),
            Tool::new(
                "bitbucket_list_branches",
                "Lists branches",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    },
                    "required": ["project_key", "repo_slug"]
                }))
            ),
            Tool::new(
                "bitbucket_create_branch",
                "Creates a new branch",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "name": {"type": "string"},
                        "start_point": {"type": "string"}
                    },
                    "required": ["project_key", "repo_slug", "name", "start_point"]
                }))
            ),
            Tool::new(
                "bitbucket_compare_branches",
                "Compares branches",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "from": {"type": "string"},
                        "to": {"type": "string"}
                    },
                    "required": ["project_key", "repo_slug", "from", "to"]
                }))
            ),
            Tool::new(
                "bitbucket_list_commits",
                "Lists commits",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    },
                    "required": ["project_key", "repo_slug"]
                }))
            ),
            Tool::new(
                "bitbucket_get_commit",
                "Gets a specific commit",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "commit_id": {"type": "string"}
                    },
                    "required": ["project_key", "repo_slug", "commit_id"]
                }))
            ),
            Tool::new(
                "bitbucket_list_files",
                "Lists files at a path",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "path": {"type": "string", "default": ""},
                        "at": {"type": "string"},
                        "start": {"type": "number", "default": 0},
                        "limit": {"type": "number", "default": 25}
                    },
                    "required": ["project_key", "repo_slug"]
                }))
            ),
            Tool::new(
                "bitbucket_get_file_content",
                "Gets file content",
                schema(json!({
                    "type": "object",
                    "properties": {
                        "project_key": {"type": "string"},
                        "repo_slug": {"type": "string"},
                        "path": {"type": "string"},
                        "at": {"type": "string"}
                    },
                    "required": ["project_key", "repo_slug", "path"]
                }))
            ),
        ];
        
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let config = config::load_from_env()
            .map_err(|e| McpError::internal_error(format!("Config error: {}", e), None))?;
        
        let checker = PermissionChecker::new(&config);
        let client = BitbucketClient::new(config.clone())
            .map_err(|e| McpError::internal_error(format!("Client error: {}", e), None))?;
        
        let args = request.arguments.unwrap_or_default();
        
        match request.name.as_ref() {
            "bitbucket_list_projects" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.list_projects(start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_get_project" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                
                let result = client.get_project(project_key).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_list_repos" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.list_repos(project_key, start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_get_repo" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                
                let result = client.get_repo(project_key, repo_slug).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_list_prs" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.list_prs(project_key, repo_slug, start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_get_pr" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let pr_id = args.get("pr_id").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing pr_id", None))?;
                
                let result = client.get_pr(project_key, repo_slug, pr_id).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_create_pr" => {
                checker.check_permission(Operation::Write)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let title = args.get("title").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing title", None))?;
                let description = args.get("description").and_then(|v| v.as_str());
                let from_branch = args.get("from_branch").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing from_branch", None))?;
                let to_branch = args.get("to_branch").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing to_branch", None))?;
                
                let reviewers = args.get("reviewers")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|name| ReviewerRequest { user: UserRequest { name: name.to_string() } })
                        .collect());
                
                let request_body = CreatePullRequestRequest {
                    title: title.to_string(),
                    description: description.map(|s| s.to_string()),
                    from_ref: RefRequest {
                        id: format!("refs/heads/{}", from_branch),
                        repository: RepositoryRequest {
                            slug: repo_slug.to_string(),
                            project: ProjectRequest { key: project_key.to_string() },
                        },
                    },
                    to_ref: RefRequest {
                        id: format!("refs/heads/{}", to_branch),
                        repository: RepositoryRequest {
                            slug: repo_slug.to_string(),
                            project: ProjectRequest { key: project_key.to_string() },
                        },
                    },
                    reviewers,
                };
                
                let result = client.create_pr(project_key, repo_slug, request_body).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_update_pr" => {
                checker.check_permission(Operation::Write)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let pr_id = args.get("pr_id").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing pr_id", None))?;
                let version = args.get("version").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing version", None))?;
                
                let request_body = UpdatePullRequestRequest {
                    version,
                    title: args.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    description: args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    reviewers: None,
                };
                
                let result = client.update_pr(project_key, repo_slug, pr_id, request_body).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_merge_pr" => {
                checker.check_permission(Operation::Write)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let pr_id = args.get("pr_id").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing pr_id", None))?;
                let version = args.get("version").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing version", None))?;
                
                let result = client.merge_pr(project_key, repo_slug, pr_id, version).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_decline_pr" => {
                checker.check_permission(Operation::Delete)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let pr_id = args.get("pr_id").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing pr_id", None))?;
                let version = args.get("version").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing version", None))?;
                
                let result = client.decline_pr(project_key, repo_slug, pr_id, version).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_pr_diff" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let pr_id = args.get("pr_id").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing pr_id", None))?;
                
                let result = client.pr_diff(project_key, repo_slug, pr_id).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(result)],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_pr_changes" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let pr_id = args.get("pr_id").and_then(|v| v.as_u64())
                    .ok_or_else(|| McpError::invalid_params("Missing pr_id", None))?;
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.pr_changes(project_key, repo_slug, pr_id, start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_list_branches" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.list_branches(project_key, repo_slug, start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_create_branch" => {
                checker.check_permission(Operation::Write)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing name", None))?;
                let start_point = args.get("start_point").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing start_point", None))?;
                
                let result = client.create_branch(project_key, repo_slug, name, start_point).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_compare_branches" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let from = args.get("from").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing from", None))?;
                let to = args.get("to").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing to", None))?;
                
                let result = client.branch_compare(project_key, repo_slug, from, to).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_list_commits" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.list_commits(project_key, repo_slug, start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_get_commit" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let commit_id = args.get("commit_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing commit_id", None))?;
                
                let result = client.get_commit(project_key, repo_slug, commit_id).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_list_files" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let at = args.get("at").and_then(|v| v.as_str());
                let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
                
                let result = client.list_files(project_key, repo_slug, path, at, start, limit).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "Serialization error".to_string()))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            "bitbucket_get_file_content" => {
                checker.check_permission(Operation::Read)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                
                let project_key = args.get("project_key").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing project_key", None))?;
                let repo_slug = args.get("repo_slug").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing repo_slug", None))?;
                let path = args.get("path").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing path", None))?;
                let at = args.get("at").and_then(|v| v.as_str());
                
                let result = client.get_file_content(project_key, repo_slug, path, at).await
                    .map_err(|e| McpError::internal_error(format!("API error: {}", e), None))?;
                
                Ok(CallToolResult {
                    content: vec![Content::text(result)],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
            _ => Err(McpError::invalid_params(format!("Unknown tool: {}", request.name), None))
        }
    }
}
