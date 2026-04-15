use std::sync::Arc;

use tokio::task::JoinSet;
use tracing::{info, warn};

use crate::agent::{Agent, AgentConfig, AgentResponse};
use crate::error::AgentError;
use mulberry_core::event::{Event, EventBus};

/// Orchestrates multiple AI agents using `tokio::task::JoinSet` for structured
/// concurrency.
///
/// Agent configs are stored centrally. Each dispatched request spawns a fresh
/// [`Agent`] inside the task so that the spawned future is `'static + Send`
/// without requiring shared mutable state.
pub struct AgentOrchestrator {
    configs: Vec<AgentConfig>,
    join_set: JoinSet<Result<AgentResponse, AgentError>>,
    event_bus: Arc<EventBus>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator connected to the given event bus.
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            configs: Vec::new(),
            join_set: JoinSet::new(),
            event_bus,
        }
    }

    /// Register an agent configuration.
    pub fn register_agent(&mut self, config: AgentConfig) {
        info!(agent_id = %config.id, name = %config.name, "Registering agent");
        self.configs.push(config);
    }

    /// Remove an agent configuration by ID.
    ///
    /// Returns `true` if the agent was found and removed.
    pub fn remove_agent(&mut self, agent_id: &str) -> bool {
        let before = self.configs.len();
        self.configs.retain(|c| c.id != agent_id);
        let removed = self.configs.len() < before;
        if removed {
            info!(agent_id, "Removed agent");
        }
        removed
    }

    /// Dispatch a prompt to a specific agent by ID.
    ///
    /// The agent task is spawned into the internal `JoinSet`. Use
    /// [`poll_results`](Self::poll_results) or [`wait_next`](Self::wait_next)
    /// to retrieve results.
    pub fn dispatch(&mut self, agent_id: &str, prompt: String) -> Result<(), AgentError> {
        let config = self
            .configs
            .iter()
            .find(|c| c.id == agent_id)
            .ok_or_else(|| AgentError::NotFound(agent_id.to_string()))?
            .clone();

        let event_bus = Arc::clone(&self.event_bus);

        // Clone AgentConfig into the spawned task and create the Agent inside
        // the task. This keeps the future `'static + Send`.
        self.join_set.spawn(async move {
            let mut agent = Agent::new(config);
            let response = agent.query(&prompt).await?;

            event_bus.publish(Event::AgentMessage {
                agent_id: response.agent_id.clone(),
                content: response.content.clone(),
            });

            Ok(response)
        });

        Ok(())
    }

    /// Dispatch a prompt to every registered agent.
    pub fn dispatch_all(&mut self, prompt: String) -> Result<(), AgentError> {
        let ids: Vec<String> = self.configs.iter().map(|c| c.id.clone()).collect();

        if ids.is_empty() {
            warn!("dispatch_all called with no registered agents");
        }

        for id in ids {
            self.dispatch(&id, prompt.clone())?;
        }

        Ok(())
    }

    /// Drain all currently completed results from the `JoinSet`.
    ///
    /// Tasks that are still running are not awaited.
    pub fn poll_results(&mut self) -> Vec<Result<AgentResponse, AgentError>> {
        let mut results = Vec::new();
        while let Some(res) = self.join_set.try_join_next() {
            results.push(flatten_join_result(res));
        }
        results
    }

    /// Wait for the next agent task to complete.
    ///
    /// Returns `None` if there are no pending tasks.
    pub async fn wait_next(&mut self) -> Option<Result<AgentResponse, AgentError>> {
        self.join_set.join_next().await.map(flatten_join_result)
    }

    /// Abort all pending agent tasks.
    pub fn abort_all(&mut self) {
        self.join_set.abort_all();
        info!("Aborted all pending agent tasks");
    }

    /// Number of in-flight agent tasks.
    pub fn pending_count(&self) -> usize {
        self.join_set.len()
    }
}

/// Convert a `JoinError` (panic / cancellation) into an `AgentError`.
fn flatten_join_result(
    res: Result<Result<AgentResponse, AgentError>, tokio::task::JoinError>,
) -> Result<AgentResponse, AgentError> {
    match res {
        Ok(inner) => inner,
        Err(join_err) => {
            if join_err.is_cancelled() {
                Err(AgentError::Cancelled)
            } else {
                Err(AgentError::InvalidResponse(format!(
                    "agent task panicked: {join_err}"
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentRole;

    fn test_config(id: &str) -> AgentConfig {
        AgentConfig {
            id: id.to_string(),
            name: format!("Test Agent {id}"),
            role: AgentRole::Assistant,
            endpoint: "http://localhost:1234/v1/chat/completions".into(),
            model: None,
            system_prompt: None,
            max_tokens: 256,
            temperature: 0.7,
        }
    }

    #[test]
    fn register_and_remove_agents() {
        let event_bus = Arc::new(EventBus::new());
        let mut orch = AgentOrchestrator::new(event_bus);

        orch.register_agent(test_config("a1"));
        orch.register_agent(test_config("a2"));
        assert_eq!(orch.configs.len(), 2);

        assert!(orch.remove_agent("a1"));
        assert_eq!(orch.configs.len(), 1);
        assert_eq!(orch.configs[0].id, "a2");

        // Removing a non-existent agent returns false.
        assert!(!orch.remove_agent("does-not-exist"));
    }

    #[test]
    fn dispatch_unknown_agent_returns_not_found() {
        let event_bus = Arc::new(EventBus::new());
        let mut orch = AgentOrchestrator::new(event_bus);

        let result = orch.dispatch("missing", "hello".into());
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentError::NotFound(id) => assert_eq!(id, "missing"),
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }

    #[test]
    fn pending_count_starts_at_zero() {
        let event_bus = Arc::new(EventBus::new());
        let orch = AgentOrchestrator::new(event_bus);
        assert_eq!(orch.pending_count(), 0);
    }
}
