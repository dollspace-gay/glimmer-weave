//! Lifetime Checker
//!
//! Ensures that references do not outlive the data they point to.
//! This prevents dangling pointers and use-after-free errors.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use crate::ast::{AstNode, Lifetime, TypeAnnotation};
use crate::source_location::SourceSpan;

/// Errors that can occur during lifetime checking
#[derive(Debug, Clone, PartialEq)]
pub enum LifetimeError {
    /// Reference outlives the data it points to
    OutlivesReferent {
        reference: String,
        reference_lifetime: String,
        referent_lifetime: String,
        span: SourceSpan,
    },
    /// Returning a reference to a local variable
    ReturnsLocalReference {
        variable: String,
        span: SourceSpan,
    },
    /// Lifetime parameter not declared
    UndeclaredLifetime {
        lifetime: String,
        span: SourceSpan,
    },
    /// Conflicting lifetime requirements
    LifetimeConflict {
        first: String,
        second: String,
        span: SourceSpan,
    },
}

impl fmt::Display for LifetimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifetimeError::OutlivesReferent {
                reference,
                reference_lifetime,
                referent_lifetime,
                span,
            } => {
                write!(
                    f,
                    "Reference '{}' with lifetime '{}' outlives referent with lifetime '{}'\n  at: {}",
                    reference, reference_lifetime, referent_lifetime, span
                )
            }
            LifetimeError::ReturnsLocalReference { variable, span } => {
                write!(
                    f,
                    "Cannot return reference to local variable '{}'\n  at: {}",
                    variable, span
                )
            }
            LifetimeError::UndeclaredLifetime { lifetime, span } => {
                write!(
                    f,
                    "Lifetime '{}' is not declared\n  at: {}",
                    lifetime, span
                )
            }
            LifetimeError::LifetimeConflict { first, second, span } => {
                write!(
                    f,
                    "Conflicting lifetime requirements: '{}' and '{}'\n  at: {}",
                    first, second, span
                )
            }
        }
    }
}

/// Tracks lifetime information for variables
#[derive(Debug, Clone)]
struct LifetimeInfo {
    /// The lifetime of this variable
    lifetime: Option<Lifetime>,
    /// Where this variable was declared (will be used for better error messages in the future)
    #[allow(dead_code)]
    span: SourceSpan,
}

/// Lifetime checker for Glimmer-Weave
pub struct LifetimeChecker {
    /// Declared lifetimes in current scope
    declared_lifetimes: Vec<String>,
    /// Lifetime information for each variable
    variables: BTreeMap<String, LifetimeInfo>,
    /// Errors found during checking
    errors: Vec<LifetimeError>,
}

impl LifetimeChecker {
    /// Create a new lifetime checker
    pub fn new() -> Self {
        LifetimeChecker {
            declared_lifetimes: Vec::new(),
            variables: BTreeMap::new(),
            errors: Vec::new(),
        }
    }

    /// Check a list of AST nodes for lifetime errors
    pub fn check(&mut self, nodes: &[AstNode]) -> Result<(), Vec<LifetimeError>> {
        for node in nodes {
            self.check_node(node);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_node(&mut self, node: &AstNode) {
        match node {
            AstNode::BindStmt { name, typ, value, span } => {
                self.check_node(value);

                let lifetime = typ.as_ref().and_then(|t| self.extract_lifetime(t));
                self.variables.insert(
                    name.clone(),
                    LifetimeInfo {
                        lifetime,
                        span: span.clone(),
                    },
                );
            }
            AstNode::WeaveStmt { name, typ, value, span } => {
                self.check_node(value);

                let lifetime = typ.as_ref().and_then(|t| self.extract_lifetime(t));
                self.variables.insert(
                    name.clone(),
                    LifetimeInfo {
                        lifetime,
                        span: span.clone(),
                    },
                );
            }
            AstNode::ChantDef {
                lifetime_params,
                params,
                body,
                return_type,
                ..
            } => {
                // Declare lifetimes for this function
                for lifetime in lifetime_params {
                    self.declared_lifetimes.push(lifetime.name.clone());
                }

                // Check parameters
                for param in params {
                    if let Some(ref typ) = param.typ {
                        self.check_type_annotation(typ);
                    }
                    if let Some(ref lifetime) = param.lifetime {
                        if !self.is_lifetime_declared(&lifetime.name) {
                            self.errors.push(LifetimeError::UndeclaredLifetime {
                                lifetime: lifetime.name.clone(),
                                span: SourceSpan::unknown(),
                            });
                        }
                    }
                }

                // Check return type
                if let Some(ref ret_typ) = return_type {
                    self.check_type_annotation(ret_typ);
                }

                // Check body
                for node in body {
                    self.check_node(node);
                }

                // Remove declared lifetimes
                self.declared_lifetimes
                    .retain(|l| !lifetime_params.iter().any(|lp| lp.name == *l));
            }
            AstNode::YieldStmt { value, span } => {
                // Check if returning a reference to a local variable
                if let AstNode::Ident { name, .. } = &**value {
                    if let Some(info) = self.variables.get(name) {
                        if info.lifetime.is_some() {
                            // This is a simplified check - in reality we'd need to verify
                            // the lifetime actually outlives the function
                            self.errors.push(LifetimeError::ReturnsLocalReference {
                                variable: name.clone(),
                                span: span.clone(),
                            });
                        }
                    }
                }
                self.check_node(value);
            }
            // Recursively check other node types
            AstNode::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.check_node(condition);
                for node in then_branch {
                    self.check_node(node);
                }
                if let Some(else_branch) = else_branch {
                    for node in else_branch {
                        self.check_node(node);
                    }
                }
            }
            AstNode::WhileStmt { condition, body, .. } => {
                self.check_node(condition);
                for node in body {
                    self.check_node(node);
                }
            }
            AstNode::BinaryOp { left, right, .. } => {
                self.check_node(left);
                self.check_node(right);
            }
            AstNode::Call { callee, args, .. } => {
                self.check_node(callee);
                for arg in args {
                    self.check_node(arg);
                }
            }
            // Literals and simple nodes don't need checking
            _ => {}
        }
    }

    fn check_type_annotation(&mut self, typ: &TypeAnnotation) {
        match typ {
            TypeAnnotation::Borrowed { lifetime, inner, .. } => {
                if let Some(ref lt) = lifetime {
                    if !self.is_lifetime_declared(&lt.name) {
                        self.errors.push(LifetimeError::UndeclaredLifetime {
                            lifetime: lt.name.clone(),
                            span: SourceSpan::unknown(),
                        });
                    }
                }
                self.check_type_annotation(inner);
            }
            TypeAnnotation::List(inner) => {
                self.check_type_annotation(inner);
            }
            TypeAnnotation::Parametrized { type_args, .. } => {
                for arg in type_args {
                    self.check_type_annotation(arg);
                }
            }
            TypeAnnotation::Function {
                param_types,
                return_type,
            } => {
                for param in param_types {
                    self.check_type_annotation(param);
                }
                self.check_type_annotation(return_type);
            }
            TypeAnnotation::Optional(inner) => {
                self.check_type_annotation(inner);
            }
            _ => {}
        }
    }

    fn extract_lifetime(&self, typ: &TypeAnnotation) -> Option<Lifetime> {
        match typ {
            TypeAnnotation::Borrowed { lifetime, .. } => lifetime.clone(),
            _ => None,
        }
    }

    fn is_lifetime_declared(&self, name: &str) -> bool {
        name == "static" || self.declared_lifetimes.contains(&name.to_string())
    }
}

impl Default for LifetimeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_checker_basic() {
        let mut checker = LifetimeChecker::new();
        let nodes = vec![
            AstNode::BindStmt {
                name: "x".to_string(),
                typ: None,
                value: Box::new(AstNode::Number {
                    value: 42.0,
                    span: SourceSpan::unknown(),
                }),
                span: SourceSpan::unknown(),
            },
        ];

        assert!(checker.check(&nodes).is_ok());
    }
}
