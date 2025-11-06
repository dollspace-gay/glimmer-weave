/// Type Errors with Natural Language Messages
///
/// Type errors in Glimmer-Weave use natural language to make them helpful
/// and understandable, not cryptic.
///
/// ## Design Philosophy
///
/// Error messages should:
/// - Use natural language, not jargon
/// - Explain what went wrong clearly
/// - Suggest how to fix it when possible
/// - Show relevant source context

use crate::type_inference::{InferType, TypeVar};
use crate::type_inference::requirement::SourceLocation;
use std::fmt;

/// Type errors that can occur during inference
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    /// Types don't match
    Mismatch {
        expected: InferType,
        got: InferType,
        location: SourceLocation,
    },

    /// Infinite type (occurs check failure)
    InfiniteType {
        var: TypeVar,
        ty: InferType,
        location: SourceLocation,
    },

    /// Variable not found in environment
    UndefinedVariable {
        name: String,
        location: SourceLocation,
    },

    /// Function called with wrong number of arguments
    ArityMismatch {
        expected: usize,
        got: usize,
        location: SourceLocation,
    },

    /// Tried to unify incompatible type constructors
    IncompatibleConstructors {
        lhs: String,
        rhs: String,
        location: SourceLocation,
    },

    /// Unsolved type variable remains after inference
    UnsolvedVariable {
        var: TypeVar,
        location: SourceLocation,
    },
}

impl TypeError {
    /// Format error with natural language message
    pub fn format_message(&self) -> String {
        match self {
            TypeError::Mismatch {
                expected,
                got,
                location,
            } => {
                format!(
                    "Type mismatch at {}:\n  Expected: {}\n  But got:  {}\n",
                    location,
                    expected.display_natural(),
                    got.display_natural()
                )
            }

            TypeError::InfiniteType { var, ty, location } => {
                format!(
                    "Cannot create infinite type at {}:\n  {} would contain itself in {}\n\n\
                    This usually means there's a circular reference in your types.",
                    location,
                    var.name(),
                    ty.display_natural()
                )
            }

            TypeError::UndefinedVariable { name, location } => {
                format!(
                    "Undefined variable '{}' at {}\n\n\
                    This variable hasn't been bound yet.",
                    name, location
                )
            }

            TypeError::ArityMismatch {
                expected,
                got,
                location,
            } => {
                format!(
                    "Wrong number of arguments at {}:\n  Expected: {} argument{}\n  But got:  {} argument{}\n",
                    location,
                    expected,
                    if *expected == 1 { "" } else { "s" },
                    got,
                    if *got == 1 { "" } else { "s" }
                )
            }

            TypeError::IncompatibleConstructors { lhs, rhs, location } => {
                format!(
                    "Cannot match {} with {} at {}:\n\n\
                    These are different types that can't be unified.",
                    lhs, rhs, location
                )
            }

            TypeError::UnsolvedVariable { var, location } => {
                format!(
                    "Could not determine type for {} at {}:\n\n\
                    Try adding a type annotation to help the inference.",
                    var.name(),
                    location
                )
            }
        }
    }

    /// Get source location for this error
    pub fn location(&self) -> &SourceLocation {
        match self {
            TypeError::Mismatch { location, .. }
            | TypeError::InfiniteType { location, .. }
            | TypeError::UndefinedVariable { location, .. }
            | TypeError::ArityMismatch { location, .. }
            | TypeError::IncompatibleConstructors { location, .. }
            | TypeError::UnsolvedVariable { location, .. } => location,
        }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_message())
    }
}

impl std::error::Error for TypeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::Type;

    #[test]
    fn test_mismatch_error() {
        let error = TypeError::Mismatch {
            expected: InferType::Concrete(Type::Number),
            got: InferType::Concrete(Type::Text),
            location: SourceLocation::new(5, 10),
        };

        let msg = error.format_message();
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("number"));
        assert!(msg.contains("text"));
        assert!(msg.contains("line 5"));
    }

    #[test]
    fn test_infinite_type_error() {
        let alpha = TypeVar::fresh(0);
        let error = TypeError::InfiniteType {
            var: alpha.clone(),
            ty: InferType::arrow(
                InferType::Var(alpha),
                InferType::Concrete(Type::Number),
            ),
            location: SourceLocation::new(8, 5),
        };

        let msg = error.format_message();
        assert!(msg.contains("infinite type"));
        assert!(msg.contains("α"));
        assert!(msg.contains("circular"));
    }

    #[test]
    fn test_undefined_variable_error() {
        let error = TypeError::UndefinedVariable {
            name: "foo".to_string(),
            location: SourceLocation::new(3, 7),
        };

        let msg = error.format_message();
        assert!(msg.contains("Undefined variable"));
        assert!(msg.contains("foo"));
        assert!(msg.contains("line 3"));
    }

    #[test]
    fn test_arity_mismatch_error() {
        let error = TypeError::ArityMismatch {
            expected: 2,
            got: 3,
            location: SourceLocation::new(10, 15),
        };

        let msg = error.format_message();
        assert!(msg.contains("Wrong number of arguments"));
        assert!(msg.contains("2 arguments"));
        assert!(msg.contains("3 arguments"));
    }

    #[test]
    fn test_error_location() {
        let error = TypeError::Mismatch {
            expected: InferType::Concrete(Type::Number),
            got: InferType::Concrete(Type::Text),
            location: SourceLocation::new(5, 10),
        };

        let loc = error.location();
        assert_eq!(loc.line, 5);
        assert_eq!(loc.column, 10);
    }
}
