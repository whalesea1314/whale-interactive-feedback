//! MCP Server binary entry point
//!
//! This binary runs the MCP server that communicates with AI assistants.

use whale_interactive_feedback_lib::mcp_server::run_mcp_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger (to stderr so it doesn't interfere with MCP protocol)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Starting Whale Interactive Feedback MCP Server...");

    // Run the MCP server
    run_mcp_server().await?;

    Ok(())
}
