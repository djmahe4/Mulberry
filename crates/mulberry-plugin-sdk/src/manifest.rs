use serde::{Deserialize, Serialize};

use crate::plugin::PluginCategory;

/// Plugin manifest loaded from a file alongside the plugin library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub category: PluginCategory,
    pub description: String,
    /// Filename of the shared library (e.g. "my_plugin.so").
    pub library: String,
    pub min_host_version: Option<String>,
}

impl PluginManifest {
    /// Create a manifest from structured data.
    pub fn new(
        id: &str,
        name: &str,
        version: &str,
        author: &str,
        category: PluginCategory,
        description: &str,
        library: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            category,
            description: description.to_string(),
            library: library.to_string(),
            min_host_version: None,
        }
    }

    /// Set the minimum host version.
    pub fn with_min_host_version(mut self, version: &str) -> Self {
        self.min_host_version = Some(version.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_new() {
        let manifest = PluginManifest::new(
            "com.example.reverb",
            "Example Reverb",
            "1.0.0",
            "Test Author",
            PluginCategory::Effect,
            "A simple reverb plugin",
            "example_reverb.so",
        );
        assert_eq!(manifest.id, "com.example.reverb");
        assert_eq!(manifest.name, "Example Reverb");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.library, "example_reverb.so");
        assert!(manifest.min_host_version.is_none());
    }

    #[test]
    fn test_manifest_with_min_host_version() {
        let manifest = PluginManifest::new(
            "com.example.synth",
            "Example Synth",
            "0.1.0",
            "Test Author",
            PluginCategory::Instrument,
            "A test synth",
            "example_synth.so",
        )
        .with_min_host_version("0.1.0");

        assert_eq!(manifest.min_host_version, Some("0.1.0".to_string()));
    }
}
