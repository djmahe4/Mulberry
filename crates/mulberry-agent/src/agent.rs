use serde::{Deserialize, Serialize};

use crate::error::AgentError;

/// Role of an AI agent in the DAW.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    /// Generates musical patterns and melodies.
    Composer,
    /// Suggests mixing and effects parameters.
    MixEngineer,
    /// Generates live-code pattern strings.
    LiveCoder,
    /// Provides music theory explanations.
    Tutor,
    /// General-purpose assistant.
    Assistant,
}

/// Configuration for a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub endpoint: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f64,
}

/// Represents an active AI agent instance.
pub struct Agent {
    pub config: AgentConfig,
    client: reqwest::Client,
    conversation_history: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Response from an agent query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub agent_id: String,
    pub content: String,
    pub role: AgentRole,
}

/// OpenAI-compatible request body.
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f64,
}

/// Expected response shape from an OpenAI-compatible API.
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

impl Agent {
    /// Create a new agent with the given configuration.
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            conversation_history: Vec::new(),
        }
    }

    /// Send a prompt to the agent's LLM endpoint and return the response.
    ///
    /// The prompt is appended to the conversation history so that multi-turn
    /// interactions are supported. The agent uses an OpenAI-compatible chat
    /// completions format.
    pub async fn query(&mut self, prompt: &str) -> Result<AgentResponse, AgentError> {
        if self.config.endpoint.is_empty() {
            return Err(AgentError::NoEndpoint);
        }

        // Build the full message list: optional system prompt + history + new user message.
        let mut messages = Vec::new();

        if let Some(system) = &self.config.system_prompt {
            messages.push(Message {
                role: "system".into(),
                content: system.clone(),
            });
        }

        messages.extend(self.conversation_history.clone());

        let user_msg = Message {
            role: "user".into(),
            content: prompt.to_string(),
        };
        messages.push(user_msg.clone());

        let request_body = ChatRequest {
            model: self
                .config
                .model
                .clone()
                .unwrap_or_else(|| "gpt-3.5-turbo".into()),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let response = self
            .client
            .post(&self.config.endpoint)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        let chat_resp: ChatResponse = response.json().await?;

        let assistant_content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| {
                AgentError::InvalidResponse("no choices returned in response".into())
            })?;

        // Record the exchange in conversation history.
        self.conversation_history.push(user_msg);
        self.conversation_history.push(Message {
            role: "assistant".into(),
            content: assistant_content.clone(),
        });

        Ok(AgentResponse {
            agent_id: self.config.id.clone(),
            content: assistant_content,
            role: self.config.role.clone(),
        })
    }

    /// Clear the conversation history.
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    /// Returns the agent's unique identifier.
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Returns the agent's role.
    pub fn role(&self) -> &AgentRole {
        &self.config.role
    }
}
