use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub verify_ssl: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub values: Vec<T>,
    #[serde(rename = "nextPageStart")]
    pub next_page_start: Option<u32>,
    #[serde(rename = "isLastPage")]
    pub is_last_page: bool,
}
