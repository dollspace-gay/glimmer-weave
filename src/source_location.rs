/// Source Location Tracking
///
/// Tracks positions in source code for error reporting.
use alloc::string::String;
use core::fmt;

/// Represents a position in source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Optional source file name
    pub file: Option<String>,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize) -> Self {
        SourceLocation {
            line,
            column,
            file: None,
        }
    }

    /// Create a source location with a file name
    pub fn with_file(line: usize, column: usize, file: String) -> Self {
        SourceLocation {
            line,
            column,
            file: Some(file),
        }
    }

    /// Create an unknown/synthetic location
    pub fn unknown() -> Self {
        SourceLocation {
            line: 0,
            column: 0,
            file: None,
        }
    }

    /// Check if this is a real location (not synthetic)
    pub fn is_known(&self) -> bool {
        self.line > 0 && self.column > 0
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_known() {
            return write!(f, "<unknown>");
        }

        if let Some(ref file) = self.file {
            write!(f, "{}:{}:{}", file, self.line, self.column)
        } else {
            write!(f, "line {}:{}", self.line, self.column)
        }
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self::unknown()
    }
}

/// Represents a span of source code (start to end)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    /// Create a new source span
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        SourceSpan { start, end }
    }

    /// Create a single-point span
    pub fn point(loc: SourceLocation) -> Self {
        SourceSpan {
            start: loc.clone(),
            end: loc,
        }
    }

    /// Create an unknown span
    pub fn unknown() -> Self {
        SourceSpan {
            start: SourceLocation::unknown(),
            end: SourceLocation::unknown(),
        }
    }

    /// Check if this span is known
    pub fn is_known(&self) -> bool {
        self.start.is_known() && self.end.is_known()
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_known() {
            return write!(f, "<unknown>");
        }

        if self.start == self.end {
            write!(f, "{}", self.start)
        } else if self.start.line == self.end.line {
            // Same line
            if let Some(ref file) = self.start.file {
                write!(f, "{}:{}:{}-{}", file, self.start.line, self.start.column, self.end.column)
            } else {
                write!(f, "line {}:{}-{}", self.start.line, self.start.column, self.end.column)
            }
        } else {
            // Different lines
            write!(f, "{} to {}", self.start, self.end)
        }
    }
}

impl Default for SourceSpan {
    fn default() -> Self {
        Self::unknown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(5, 10);
        assert_eq!(loc.to_string(), "line 5:10");

        let loc_with_file = SourceLocation::with_file(5, 10, "test.gw".to_string());
        assert_eq!(loc_with_file.to_string(), "test.gw:5:10");

        let unknown = SourceLocation::unknown();
        assert_eq!(unknown.to_string(), "<unknown>");
    }

    #[test]
    fn test_source_span_display() {
        let start = SourceLocation::new(5, 10);
        let end = SourceLocation::new(5, 20);
        let span = SourceSpan::new(start, end);
        assert_eq!(span.to_string(), "line 5:10-20");

        let start = SourceLocation::new(5, 10);
        let end = SourceLocation::new(8, 5);
        let span = SourceSpan::new(start, end);
        assert_eq!(span.to_string(), "line 5:10 to line 8:5");
    }

    #[test]
    fn test_source_location_known() {
        assert!(SourceLocation::new(1, 1).is_known());
        assert!(!SourceLocation::unknown().is_known());
    }
}
