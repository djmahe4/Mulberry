// CONTEXT7 REVIEW:
// Problem: Plugin trait must be object-safe AND real-time safe
// Decision: Block-based processing with &[f32] -> &mut [f32], no generics in trait
// Why this is correct: Object-safe trait allows Box<dyn Plugin>. Block processing
//   matches DAW buffer model. No allocation in process_block enforced by contract.

/// Parameter of a plugin with min/max/default/current value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginParameter {
    pub id: String,
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

/// Metadata about a plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

/// Core plugin trait for all built-in and external audio effects.
///
/// Implementors MUST NOT allocate on the heap in `process_block`.
/// All buffers and state must be pre-allocated at creation time.
pub trait Plugin: Send {
    /// Process a mono audio block in-place.
    ///
    /// # REALTIME SAFE: no allocation, no locks
    fn process_block(&mut self, input: &[f32], output: &mut [f32]);

    /// Return plugin metadata.
    fn info(&self) -> PluginInfo;

    /// Return all parameters.
    fn parameters(&self) -> Vec<PluginParameter>;

    /// Set a parameter by its id string.
    fn set_parameter(&mut self, id: &str, value: f32);

    /// Get a parameter value by id.
    fn get_parameter(&self, id: &str) -> Option<f32>;

    /// Called when the sample rate changes. Re-initializes DSP state.
    fn set_sample_rate(&mut self, sample_rate: f32);

    /// Reset all plugin state (e.g., clear delay lines, reset envelopes).
    fn reset(&mut self);
}
