use bitbucket_mcp::mcp::BitbucketMcpServer;
use rmcp::{ServiceExt, transport::stdio};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = BitbucketMcpServer::new();
    let transport = stdio();
    let server_handle = server.serve(transport).await?;
    let quit_reason = server_handle.waiting().await?;
    
    eprintln!("Server shutdown: {:?}", quit_reason);
    
    Ok(())
}
