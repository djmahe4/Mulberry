//! Plugin panel state.

use serde::{Deserialize, Serialize};

/// A plugin instance in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstance {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub parameters: Vec<PluginParamView>,
}

/// A single plugin parameter view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParamView {
    pub id: String,
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

/// State for the plugin panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPanelState {
    pub instances: Vec<PluginInstance>,
    pub selected: Option<String>,
}

impl Default for PluginPanelState {
    fn default() -> Self {
        let instances = vec![
            PluginInstance { id: "eq".into(), name: "8-Band EQ".into(), enabled: false, parameters: Vec::new() },
            PluginInstance { id: "compressor".into(), name: "Compressor".into(), enabled: false, parameters: Vec::new() },
            PluginInstance { id: "reverb".into(), name: "Reverb".into(), enabled: false, parameters: Vec::new() },
            PluginInstance { id: "delay".into(), name: "Delay".into(), enabled: false, parameters: Vec::new() },
            PluginInstance { id: "drum_enhancer".into(), name: "Drum Enhancer".into(), enabled: false, parameters: Vec::new() },
            PluginInstance { id: "bass_enhancer".into(), name: "Bass Enhancer".into(), enabled: false, parameters: Vec::new() },
        ];
        Self { instances, selected: None }
    }
}
