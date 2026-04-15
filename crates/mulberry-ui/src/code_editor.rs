//! Code editor panel state (center panel).

use serde::{Deserialize, Serialize};

/// State for the Monaco/CodeMirror code editor.
///
/// # CONTEXT7 REVIEW:
/// Problem: Live code evaluation must not block the UI thread
/// Decision: Code evaluation is dispatched as a UiCommand::EvalCode, handled async in backend
/// Why this is correct: The UI remains responsive; errors are returned via UiResponse::Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEditorState {
    pub content: String,
    pub language: EditorLanguage,
    pub cursor_line: u32,
    pub cursor_col: u32,
    pub errors: Vec<CodeError>,
    pub eval_on_change: bool,
}

/// Editor language mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorLanguage {
    MulberryPattern,
    Rust,
    Javascript,
}

/// A code error annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeError {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

impl Default for CodeEditorState {
    fn default() -> Self {
        Self {
            content: r#"// Welcome to Mulberry Live Code!
// Type patterns below and press Ctrl+Enter to evaluate.

"c4 e4 g4 c5"
"#
            .to_string(),
            language: EditorLanguage::MulberryPattern,
            cursor_line: 0,
            cursor_col: 0,
            errors: Vec::new(),
            eval_on_change: false,
        }
    }
}
