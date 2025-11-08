//! Error Formatter
//!
//! Pretty-prints diagnostics with source location information.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

use crate::source_location::SourceSpan;

/// Severity level of a diagnostic message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error: compilation cannot continue
    Error,
    /// Warning: potential issue but compilation can continue
    Warning,
    /// Info: informational message
    Info,
    /// Help: suggestion for fixing an error
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Help => write!(f, "help"),
        }
    }
}

/// A label pointing to a specific location in source code
#[derive(Debug, Clone)]
pub struct Label {
    /// The source location this label points to
    pub span: SourceSpan,
    /// The message for this label
    pub message: Option<String>,
    /// Whether this is the primary label (highlighted differently)
    pub primary: bool,
}

impl Label {
    /// Create a new primary label at the given span
    pub fn primary(span: SourceSpan, message: impl Into<String>) -> Self {
        Label {
            span,
            message: Some(message.into()),
            primary: true,
        }
    }

    /// Create a new secondary label at the given span
    pub fn secondary(span: SourceSpan, message: impl Into<String>) -> Self {
        Label {
            span,
            message: Some(message.into()),
            primary: false,
        }
    }
}

/// A diagnostic message with source location information
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity of this diagnostic
    pub severity: Severity,
    /// Main error message
    pub message: String,
    /// Labels pointing to relevant source locations
    pub labels: Vec<Label>,
    /// Additional notes or suggestions
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Create a new warning diagnostic
    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Add a primary label to this diagnostic
    pub fn with_primary_label(mut self, span: SourceSpan, message: impl Into<String>) -> Self {
        self.labels.push(Label::primary(span, message));
        self
    }

    /// Add a secondary label to this diagnostic
    pub fn with_secondary_label(mut self, span: SourceSpan, message: impl Into<String>) -> Self {
        self.labels.push(Label::secondary(span, message));
        self
    }

    /// Add a note to this diagnostic
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Format this diagnostic for display
    pub fn format(&self) -> String {
        let mut output = format!("{}: {}\n", self.severity, self.message);

        // Add labels
        for label in &self.labels {
            let marker = if label.primary { "--->" } else { "----" };
            output.push_str(&format!("  {} {}", marker, label.span));
            if let Some(ref msg) = label.message {
                output.push_str(&format!(": {}", msg));
            }
            output.push('\n');
        }

        // Add notes
        for note in &self.notes {
            output.push_str(&format!("  = note: {}\n", note));
        }

        output
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_location::SourceLocation;

    #[test]
    fn test_diagnostic_format() {
        let span = SourceSpan::point(SourceLocation::new(10, 5));
        let diag = Diagnostic::error("Use of moved value 'x'")
            .with_primary_label(span, "value used here")
            .with_note("'x' was moved on line 8");

        let formatted = diag.format();
        assert!(formatted.contains("error: Use of moved value 'x'"));
        assert!(formatted.contains("line 10:5"));
        assert!(formatted.contains("value used here"));
        assert!(formatted.contains("note: 'x' was moved on line 8"));
    }
}
