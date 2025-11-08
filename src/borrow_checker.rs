//! Borrow Checker
//!
//! Ensures memory safety through Rust-style borrow checking rules:
//! - At any given time, you can have either one mutable reference or any number of immutable references
//! - References must always be valid
//! - Values cannot be used after being moved

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

use crate::ast::{AstNode, BorrowMode};
use crate::source_location::SourceSpan;

/// Errors that can occur during borrow checking
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowError {
    /// Variable used after being moved
    UseAfterMove {
        variable: String,
        moved_at: SourceSpan,
        used_at: SourceSpan,
    },
    /// Mutable borrow while immutable borrows exist
    MutableBorrowConflict {
        variable: String,
        immutable_borrow_at: SourceSpan,
        mutable_borrow_at: SourceSpan,
    },
    /// Multiple mutable borrows
    MultipleMutableBorrows {
        variable: String,
        first_borrow_at: SourceSpan,
        second_borrow_at: SourceSpan,
    },
    /// Borrowing a moved value
    BorrowOfMovedValue {
        variable: String,
        moved_at: SourceSpan,
        borrowed_at: SourceSpan,
    },
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::UseAfterMove { variable, moved_at, used_at } => {
                write!(
                    f,
                    "Use of moved value '{}'\n  moved at: {}\n  used at: {}",
                    variable, moved_at, used_at
                )
            }
            BorrowError::MutableBorrowConflict {
                variable,
                immutable_borrow_at,
                mutable_borrow_at,
            } => {
                write!(
                    f,
                    "Cannot borrow '{}' as mutable because it is also borrowed as immutable\n  immutable borrow at: {}\n  mutable borrow at: {}",
                    variable, immutable_borrow_at, mutable_borrow_at
                )
            }
            BorrowError::MultipleMutableBorrows {
                variable,
                first_borrow_at,
                second_borrow_at,
            } => {
                write!(
                    f,
                    "Cannot borrow '{}' as mutable more than once\n  first borrow at: {}\n  second borrow at: {}",
                    variable, first_borrow_at, second_borrow_at
                )
            }
            BorrowError::BorrowOfMovedValue {
                variable,
                moved_at,
                borrowed_at,
            } => {
                write!(
                    f,
                    "Cannot borrow '{}' because it was moved\n  moved at: {}\n  borrow attempted at: {}",
                    variable, moved_at, borrowed_at
                )
            }
        }
    }
}

/// Tracks the state of a variable in the borrow checker
#[derive(Debug, Clone, PartialEq)]
enum VarState {
    /// Variable is owned and can be used
    Owned,
    /// Variable was moved
    Moved(SourceSpan),
    /// Variable is immutably borrowed
    ImmutablyBorrowed(Vec<SourceSpan>),
    /// Variable is mutably borrowed
    MutablyBorrowed(SourceSpan),
}

/// Borrow checker for Glimmer-Weave
pub struct BorrowChecker {
    /// Current state of each variable
    variables: BTreeMap<String, VarState>,
    /// Errors found during checking
    errors: Vec<BorrowError>,
}

impl BorrowChecker {
    /// Create a new borrow checker
    pub fn new() -> Self {
        BorrowChecker {
            variables: BTreeMap::new(),
            errors: Vec::new(),
        }
    }

    /// Check a list of AST nodes for borrow errors
    pub fn check(&mut self, nodes: &[AstNode]) -> Result<(), Vec<BorrowError>> {
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
            AstNode::BindStmt { name, value, .. } => {
                self.check_node(value);
                self.variables.insert(name.clone(), VarState::Owned);
            }
            AstNode::WeaveStmt { name, value, .. } => {
                self.check_node(value);
                self.variables.insert(name.clone(), VarState::Owned);
            }
            AstNode::SetStmt { name, value, span } => {
                // Check if variable is moved or borrowed
                if let Some(state) = self.variables.get(name) {
                    match state {
                        VarState::Moved(moved_at) => {
                            self.errors.push(BorrowError::UseAfterMove {
                                variable: name.clone(),
                                moved_at: moved_at.clone(),
                                used_at: span.clone(),
                            });
                        }
                        VarState::ImmutablyBorrowed(borrows) => {
                            if let Some(borrow_at) = borrows.first() {
                                self.errors.push(BorrowError::MutableBorrowConflict {
                                    variable: name.clone(),
                                    immutable_borrow_at: borrow_at.clone(),
                                    mutable_borrow_at: span.clone(),
                                });
                            }
                        }
                        VarState::MutablyBorrowed(borrow_at) => {
                            self.errors.push(BorrowError::MultipleMutableBorrows {
                                variable: name.clone(),
                                first_borrow_at: borrow_at.clone(),
                                second_borrow_at: span.clone(),
                            });
                        }
                        VarState::Owned => {}
                    }
                }
                self.check_node(value);
            }
            AstNode::Ident { name, span } => {
                // Check if variable is moved
                if let Some(state) = self.variables.get(name) {
                    if let VarState::Moved(moved_at) = state {
                        self.errors.push(BorrowError::UseAfterMove {
                            variable: name.clone(),
                            moved_at: moved_at.clone(),
                            used_at: span.clone(),
                        });
                    }
                }
            }
            AstNode::ChantDef { params, body, .. } => {
                // Check parameter borrow modes
                for param in params {
                    let state = match param.borrow_mode {
                        BorrowMode::Owned => VarState::Owned,
                        BorrowMode::Borrowed => VarState::ImmutablyBorrowed(Vec::new()),
                        BorrowMode::BorrowedMut => VarState::MutablyBorrowed(SourceSpan::unknown()),
                    };
                    self.variables.insert(param.name.clone(), state);
                }
                for node in body {
                    self.check_node(node);
                }
            }
            // Recursively check other node types
            AstNode::IfStmt { condition, then_branch, else_branch, .. } => {
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
            AstNode::ForStmt { body, iterable, .. } => {
                self.check_node(iterable);
                for node in body {
                    self.check_node(node);
                }
            }
            AstNode::BinaryOp { left, right, .. } => {
                self.check_node(left);
                self.check_node(right);
            }
            AstNode::UnaryOp { operand, .. } => {
                self.check_node(operand);
            }
            AstNode::Call { callee, args, .. } => {
                self.check_node(callee);
                for arg in args {
                    self.check_node(arg);
                }
            }
            AstNode::YieldStmt { value, .. } => {
                self.check_node(value);
            }
            AstNode::List { elements, .. } => {
                for elem in elements {
                    self.check_node(elem);
                }
            }
            AstNode::FieldAccess { object, .. } => {
                self.check_node(object);
            }
            AstNode::IndexAccess { object, index, .. } => {
                self.check_node(object);
                self.check_node(index);
            }
            // Literals don't need checking
            AstNode::Number { .. }
            | AstNode::Text { .. }
            | AstNode::Truth { .. }
            | AstNode::Nothing { .. } => {}
            _ => {
                // TODO: Handle other node types
            }
        }
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_checker_basic() {
        let mut checker = BorrowChecker::new();
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
