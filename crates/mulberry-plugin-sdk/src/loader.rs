use std::path::Path;

use libloading::{Library, Symbol};
use tracing::{error, info};

use crate::error::PluginError;
use crate::manifest::PluginManifest;
use crate::plugin::MulberryPlugin;

/// Type signature for the plugin creation function that plugins must export.
/// The function name must be `mulberry_plugin_create`.
pub type CreatePluginFn = unsafe fn() -> *mut dyn MulberryPlugin;

/// Loads plugins from shared libraries.
///
/// # CONTEXT7 REVIEW:
/// Issue: Dynamic library loading is inherently unsafe
/// Resolution: Wrap all unsafe code in a single well-audited loader.
///   Validate library symbols before calling. Document safety invariants.
/// Why: Centralizes unsafety for easier auditing. Plugin authors must
///   use the provided export function to ensure correct ABI.
pub struct PluginLoader {
    loaded: Vec<Option<LoadedPlugin>>,
}

struct LoadedPlugin {
    _library: Library, // Must keep library alive as long as plugin exists
    plugin: Box<dyn MulberryPlugin>,
    #[allow(dead_code)]
    manifest: PluginManifest,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            loaded: Vec::new(),
        }
    }

    /// Load a plugin from a shared library.
    ///
    /// Returns the index of the loaded plugin in the internal storage.
    pub fn load(
        &mut self,
        library_path: &Path,
        manifest: PluginManifest,
    ) -> Result<usize, PluginError> {
        // SAFETY: We trust that the plugin library correctly implements
        // the mulberry_plugin_create function with the expected signature.
        // This is enforced by the plugin SDK's export macro.
        let library = unsafe { Library::new(library_path) }.map_err(|e| {
            error!(path = %library_path.display(), error = %e, "Failed to load plugin library");
            PluginError::LoadError(format!(
                "failed to load '{}': {}",
                library_path.display(),
                e
            ))
        })?;

        let plugin = unsafe {
            let create_fn: Symbol<CreatePluginFn> =
                library.get(b"mulberry_plugin_create").map_err(|e| {
                    PluginError::SymbolNotFound(format!(
                        "mulberry_plugin_create not found in '{}': {}",
                        library_path.display(),
                        e
                    ))
                })?;

            // SAFETY: The create function must return a valid, heap-allocated
            // MulberryPlugin. This is the core trust boundary for plugins.
            let raw = create_fn();
            if raw.is_null() {
                return Err(PluginError::InitError(
                    "plugin creation function returned null".to_string(),
                ));
            }
            Box::from_raw(raw)
        };

        info!(
            plugin_id = %manifest.id,
            plugin_name = %manifest.name,
            "Plugin loaded successfully"
        );

        let index = self.loaded.len();
        self.loaded.push(Some(LoadedPlugin {
            _library: library,
            plugin,
            manifest,
        }));

        Ok(index)
    }

    /// Get a reference to a loaded plugin by index.
    pub fn get_plugin(&self, index: usize) -> Option<&dyn MulberryPlugin> {
        match self.loaded.get(index) {
            Some(Some(lp)) => Some(lp.plugin.as_ref()),
            _ => None,
        }
    }

    /// Get a mutable reference to a loaded plugin by index.
    pub fn get_plugin_mut(&mut self, index: usize) -> Option<&mut dyn MulberryPlugin> {
        match self.loaded.get_mut(index) {
            Some(Some(lp)) => Some(lp.plugin.as_mut()),
            _ => None,
        }
    }

    /// Unload a plugin by index. Calls `shutdown()` before removing.
    pub fn unload(&mut self, index: usize) -> Result<(), PluginError> {
        let slot = self
            .loaded
            .get_mut(index)
            .ok_or_else(|| PluginError::NotFound(format!("no plugin at index {}", index)))?;

        match slot.take() {
            Some(mut loaded) => {
                info!(
                    plugin_id = %loaded.manifest.id,
                    "Unloading plugin"
                );
                loaded.plugin.shutdown();
                // LoadedPlugin (and its Library) are dropped here
                Ok(())
            }
            None => Err(PluginError::NotFound(format!(
                "plugin at index {} already unloaded",
                index
            ))),
        }
    }

    /// Number of currently loaded plugins (including unloaded slots).
    pub fn loaded_count(&self) -> usize {
        self.loaded.iter().filter(|s| s.is_some()).count()
    }

    /// Scan a directory for plugin manifest files and attempt to load each.
    ///
    /// Looks for files with a `.json` extension and tries to parse them as
    /// manifests. This is a placeholder — real implementation would support
    /// TOML manifests as well.
    pub fn scan_directory(&mut self, dir: &Path) -> Vec<Result<usize, PluginError>> {
        let mut results = Vec::new();

        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                results.push(Err(PluginError::IoError(e)));
                return results;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                // In a real implementation, we would parse the manifest and
                // resolve the library path. For now, skip non-loadable manifests.
                info!(path = %path.display(), "Found potential plugin manifest");
            }
        }

        results
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_new() {
        let loader = PluginLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_plugin_loader_default() {
        let loader = PluginLoader::default();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_get_plugin_empty() {
        let loader = PluginLoader::new();
        assert!(loader.get_plugin(0).is_none());
    }

    #[test]
    fn test_get_plugin_mut_empty() {
        let mut loader = PluginLoader::new();
        assert!(loader.get_plugin_mut(0).is_none());
    }

    #[test]
    fn test_unload_nonexistent() {
        let mut loader = PluginLoader::new();
        assert!(loader.unload(0).is_err());
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let mut loader = PluginLoader::new();
        let results = loader.scan_directory(Path::new("/nonexistent/path"));
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
