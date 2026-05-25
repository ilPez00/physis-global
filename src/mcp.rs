//! MCP (Model Context Protocol) server and client — expose Physis as an
//! MCP service on `127.0.0.1:9876`, and a stub client for remote commands.

#[cfg(feature = "mcp")]
mod mcp_impl {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::actor::PDCActor;
    use crate::config::{OntologyLoader, PhysisConfig};
    use crate::dream::DreamEngine;
    use crate::mapper::OntologyMapper;
    use crate::models::Goal;

    /// Shared state served by the MCP server: mapper, actor, dream engine, goals.
    pub struct PhysisContext {
        pub mapper: OntologyMapper,
        pub actor: PDCActor,
        pub dreams: DreamEngine,
        pub goals: Vec<Goal>,
    }

    /// Thread-safe shared reference to the Physis MCP context.
    pub type SharedContext = Arc<Mutex<PhysisContext>>;

    /// Start the Physis MCP server on `127.0.0.1:9876`. Loads config, builds
    /// mapper, actor, and dream engine, then serves the MCP protocol.
    pub async fn start_mcp_server(config: PhysisConfig) -> anyhow::Result<()> {
        let ontology = OntologyLoader::load_all(&config);
        let mapper = OntologyMapper::new(ontology);
        let actor = PDCActor::new(config.pdca_stagnant_threshold, config.pdca_stagnant_window);
        let dreams = DreamEngine::new(mapper.trie.clone());

        let ctx = Arc::new(Mutex::new(PhysisContext {
            mapper,
            actor,
            dreams,
            goals: Vec::new(),
        }));

        let server = <dyn rmcp::Service>::new(ctx);

        println!("Physis MCP server starting...");
        server.serve("127.0.0.1:9876").await?;

        Ok(())
    }
}

#[cfg(feature = "mcp")]
pub use mcp_impl::*;

/// HTTP client stub for sending commands to a running Physis MCP server.
pub struct McpClient {
    #[allow(dead_code)]
    base_url: String,
}

impl McpClient {
    /// Create a new client pointing at the given server URL.
    pub fn new(url: &str) -> Self {
        Self {
            base_url: url.to_string(),
        }
    }

    /// Send a command to the MCP server. Currently returns an error (not yet implemented).
    pub async fn send_command(&self, _command: &str, _args: &[(&str, &str)]) -> anyhow::Result<String> {
        anyhow::bail!("MCP client not yet implemented; use the CLI instead")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_new() {
        let client = McpClient::new("http://localhost:9876");
        assert!(client.base_url.contains("localhost"));
    }

    #[tokio::test]
    async fn test_mcp_client_send_command_returns_error() {
        let client = McpClient::new("http://localhost:9876");
        let result = client.send_command("physis_health", &[]).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet implemented"));
    }
}
