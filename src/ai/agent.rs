//! Tool-using agent loop that orchestrates LLM calls with tool execution.
//!
//! The agent sends messages to the provider cascade, handles tool-call responses,
//! and loops until the model produces a final text response or the maximum
//! number of tool rounds is reached.

use crate::ai::provider::{Content, ContentPart, ImageUrl, Message};
use crate::ai::tools::{ToolRegistry, call_tool};
use crate::ai::{AiError, AiResult, provider::ProviderCascade};
use serde::{Deserialize, Serialize};

const MAX_TOOL_ROUNDS: u32 = 8;

/// Configuration for an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// System prompt prepended to every request.
    pub system_prompt: String,
    /// Maximum tokens for each LLM response.
    pub max_tokens: u32,
    /// Image detail level ("low", "high").
    pub detail: String,
}

/// The result of an agent run.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    /// The final text response from the LLM.
    pub text: String,
    /// Which provider served the final response.
    pub provider: String,
    /// Number of tool-call rounds executed.
    pub tool_rounds: u32,
    /// Approximate tokens used across all rounds.
    pub tokens_used: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            max_tokens: 1024,
            detail: "low".into(),
        }
    }
}

/// Run the agent loop: build messages, call provider cascade, handle tool calls.
///
/// The agent sends `query` (optionally with a base64 image `frame_b64`) to the
/// provider cascade. If the model returns tool calls, they are executed via
/// `ToolRegistry` and the results are fed back. The loop terminates when the
/// model returns a plain text response or `MAX_TOOL_ROUNDS` is exceeded.
pub async fn run_agent(
    provider_cascade: &ProviderCascade,
    tools: &ToolRegistry,
    config: &AgentConfig,
    history: &[Message],
    query: &str,
    frame_b64: Option<&str>,
    mode: &str,
    model_override: Option<&str>,
) -> AiResult<AgentOutput> {
    let mut messages: Vec<Message> = Vec::new();

    messages.push(Message {
        role: "system".into(),
        content: Content::Text(config.system_prompt.clone()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    });

    let mode_prefix = match mode {
        "SCOUT" => "[SCOUT MODE - brief observation]\n",
        "ANALYZE" => "[ANALYZE MODE - detailed analysis]\n",
        "DATA" => "[DATA MODE - structured output]\n",
        _ => "",
    };

    messages.extend_from_slice(history);

    let user_content = if let Some(b64) = frame_b64 {
        Content::Parts(vec![
            ContentPart {
                type_: "text".into(),
                text: Some(format!("{mode_prefix}{query}")),
                image_url: None,
            },
            ContentPart {
                type_: "image_url".into(),
                text: None,
                image_url: Some(ImageUrl {
                    url: format!("data:image/jpeg;base64,{b64}"),
                    detail: Some(config.detail.clone()),
                }),
            },
        ])
    } else {
        Content::Text(format!("{mode_prefix}{query}"))
    };

    messages.push(Message {
        role: "user".into(),
        content: user_content,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    });

    let route = provider_cascade.classify(query, frame_b64.is_some());
    let tool_defs = tools.definitions();
    let defs_for_call = if tool_defs.is_empty() {
        None
    } else {
        Some(tool_defs)
    };

    let mut tool_rounds = 0u32;
    let mut total_tokens = 0u32;

    loop {
        let (resp, provider_name) = if let Some(m) = model_override {
            let p = provider_cascade.providers_for_task(&route).into_iter()
                .find(|p| p.model() == m || p.name() == m)
                .ok_or_else(|| AiError::NoProvider)?;
            (p.complete(&messages, defs_for_call, Some(config.max_tokens)).await?, p.name().to_string())
        } else {
            provider_cascade
                .complete(&messages, defs_for_call, &route, Some(config.max_tokens))
                .await?
        };

        if let Some(u) = &resp.content {
            total_tokens += u.len() as u32;
        }

        messages.push(Message {
            role: "assistant".into(),
            content: Content::Text(resp.content.clone().unwrap_or_default()),
            tool_calls: resp.tool_calls.clone(),
            tool_call_id: None,
            name: None,
        });

        let calls = match resp.tool_calls {
            Some(c) if !c.is_empty() => c,
            _ => {
                let text = resp.content.unwrap_or_default();
                return Ok(AgentOutput {
                    text,
                    provider: provider_name,
                    tool_rounds,
                    tokens_used: total_tokens,
                });
            }
        };

        tool_rounds += 1;
        if tool_rounds > MAX_TOOL_ROUNDS {
            return Err(AiError::MaxToolRounds);
        }

        for call in &calls {
            let args: serde_json::Value =
                serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::json!({}));
            let result = call_tool(tools, &call.function.name, args).await;

            messages.push(Message {
                role: "tool".into(),
                content: Content::Text(result),
                tool_calls: None,
                tool_call_id: Some(call.id.clone()),
                name: Some(call.function.name.clone()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_defaults() {
        let config = AgentConfig::default();
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.detail, "low");
        assert_eq!(config.system_prompt, "");
    }

    #[test]
    fn test_agent_mode_prefixes() {
        let cases = [
            ("SCOUT", "[SCOUT MODE - brief observation]\nquery"),
            ("ANALYZE", "[ANALYZE MODE - detailed analysis]\nquery"),
            ("DATA", "[DATA MODE - structured output]\nquery"),
            ("OTHER", "query"),
        ];
        for (mode, expected) in &cases {
            let prefix = match *mode {
                "SCOUT" => "[SCOUT MODE - brief observation]\n",
                "ANALYZE" => "[ANALYZE MODE - detailed analysis]\n",
                "DATA" => "[DATA MODE - structured output]\n",
                _ => "",
            };
            assert_eq!(format!("{prefix}query"), *expected, "mode={mode}");
        }
    }

    #[test]
    fn test_agent_max_tool_rounds_constant() {
        assert_eq!(MAX_TOOL_ROUNDS, 8);
    }
}
