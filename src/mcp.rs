use std::path::PathBuf;

use crate::search::{self, lexical_search, default_index_path, SearchResult, DEFAULT_SEARCH_LIMIT, EmbeddingProvider, OllamaEmbeddingProvider};
use crate::store::AliasStore;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct McpSearchResult {
    pub alias_name: String,
    pub command: String,
    pub score: f32,
    pub reason: String,
     #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AliasSearchParams {
    pub query: String,
     #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

/// Execute an alias search and return structured results.
/// This is the core handler that both CLI and MCP call.
pub async fn handle_alias_search(
    params: AliasSearchParams,
    data_file: &PathBuf,
) -> Vec<McpSearchResult> {
    let store = load_store(data_file);
    let db_path = default_index_path();
    let db_str = db_path.to_string_lossy().to_string();

     // Try semantic search first
    let provider = OllamaEmbeddingProvider::default();
    let (results, used_fallback) = match search::search_aliases(&db_str, &params.query, &provider, params.limit).await {
        Ok(r) if !r.is_empty() => (r, false),
         _ => (lexical_search(&store, &params.query, params.limit), true),
     };

    results.into_iter().map(|r| McpSearchResult {
        alias_name: r.alias_name,
        command: r.command,
        score: r.score,
        reason: r.reason,
        warning: if used_fallback {
            Some("Semantic search unavailable, using lexical fallback".to_string())
         } else {
            None
         },
     }).collect()
}

fn load_store(path: &PathBuf) -> AliasStore {
    if let Ok(toml_content) = std::fs::read_to_string(path) {
        AliasStore::from_toml(&toml_content).unwrap_or_default()
     } else {
        AliasStore::default()
     }
}

/// Run the MCP stdio server.
///
/// This implements a minimal MCP server that exposes the `alias_search` tool.
/// It communicates over stdio using JSON-RPC as specified by MCP.
pub async fn run_mcp_server(data_file: PathBuf) {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

     // MCP handshake — initialize
    let lines = stdin.lock().lines();
    for line_result in lines {
        let line = match line_result {
            Ok(l) => l,
             _ => break,
         };

        let req: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
             _ => continue,
         };

        let method = req.get("method").and_then(|m| m.as_str());
        let id = req.get("id");

        match method {
             // Initialize handshake
            Some("initialize") => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2025-06-18",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "aliasman-mcp",
                            "version": "0.0.1"
                        }
                    }
                });
                writeln!(out, "{}", response.to_string()).ok();
             }

             // List available tools
            Some("tools/list") => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [{
                            "name": "alias_search",
                            "description": "Search aliases semantically using natural language. Returns ranked aliases matching the query intent.",
                            "inputSchema": {
                                "type": "object",
                                "required": ["query"],
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "Natural language search query"
                                    },
                                    "limit": {
                                        "type": "integer",
                                        "description": "Maximum number of results (default: 5)"
                                    }
                                }
                            }
                        }]
                    }
                });
                writeln!(out, "{}", response.to_string()).ok();
             }

             // Call a tool
            Some("tools/call") => {
                let tool_name = req.get("params").and_then(|p| p.get("name")).and_then(|n| n.as_str());
                let arguments = req.get("params").and_then(|p| p.get("arguments"));

                if tool_name == Some("alias_search") {
                    let query = arguments
                         .and_then(|a| a.get("query"))
                         .and_then(|q| q.as_str())
                         .unwrap_or("");
                    let limit = arguments
                         .and_then(|a| a.get("limit"))
                         .and_then(|l| l.as_u64())
                         .map(|l| l as usize)
                         .unwrap_or(DEFAULT_SEARCH_LIMIT);

                    let params = AliasSearchParams {
                        query: query.to_string(),
                        limit,
                     };

                    let results = handle_alias_search(params, &data_file).await;
                    let fallback = results.iter().any(|r| r.warning.is_some());

                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string(&serde_json::json!({
                                    "results": results,
                                    "count": results.len(),
                                    "fallback": fallback
                                })).unwrap_or_default()
                            }]
                        }
                    });
                    writeln!(out, "{}", response.to_string()).ok();
                 } else {
                     // Unknown tool
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("Unknown tool: {:?}", tool_name)
                        }
                    });
                    writeln!(out, "{}", response.to_string()).ok();
                 }
             }

            _ => {
                 // NOP for notifications or unknown methods
             }
         }
     }
}
