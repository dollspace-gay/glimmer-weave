//! Borrow Checker
//!
//! Ensures memory safety through Rust-style borrow checking rules:
//! - At any given time, you can have either one mutable reference or any number of immutable references
//! - References must always be valid
//! - Values cannot be used after being moved

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
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

    /// Check if a variable reference would move the value
    /// Returns the variable name if this is a move, None if it's a copy or borrow
    fn check_move(&self, node: &AstNode) -> Option<(String, SourceSpan)> {
        match node {
            // Direct variable reference - might be a move depending on type
            AstNode::Ident { name, span } => {
                // For now, assume all variables are move types except simple values
                // In a real implementation, we'd check the type system
                // Copy types: Number, Truth, Nothing
                // Move types: Text, List, Map, custom structs
                Some((name.clone(), span.clone()))
            }
            // Explicit borrow never moves
            AstNode::BorrowExpr { .. } => None,
            // Field access, index access, literals, etc. don't move variables
            _ => None,
        }
    }

    /// Mark a variable as moved
    fn mark_moved(&mut self, name: &str, span: SourceSpan) {
        if let Some(state) = self.variables.get(name) {
            // Check if already moved or borrowed
            match state {
                VarState::Moved(moved_at) => {
                    self.errors.push(BorrowError::UseAfterMove {
                        variable: name.to_string(),
                        moved_at: moved_at.clone(),
                        used_at: span.clone(),
                    });
                }
                VarState::ImmutablyBorrowed(borrows) => {
                    if let Some(borrow_at) = borrows.first() {
                        self.errors.push(BorrowError::BorrowOfMovedValue {
                            variable: name.to_string(),
                            moved_at: borrow_at.clone(),
                            borrowed_at: span.clone(),
                        });
                    }
                }
                VarState::MutablyBorrowed(borrow_at) => {
                    self.errors.push(BorrowError::BorrowOfMovedValue {
                        variable: name.to_string(),
                        moved_at: borrow_at.clone(),
                        borrowed_at: span.clone(),
                    });
                }
                VarState::Owned => {
                    // Mark as moved
                    self.variables.insert(name.to_string(), VarState::Moved(span));
                }
            }
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
            AstNode::BindStmt { name, typ: _, value, span } => {
                // Check if the value is being moved
                if let Some((moved_var, _)) = self.check_move(value) {
                    self.mark_moved(&moved_var, span.clone());
                } else {
                    self.check_node(value);
                }
                // New variable takes ownership
                self.variables.insert(name.clone(), VarState::Owned);
            }
            AstNode::WeaveStmt { name, typ: _, value, span } => {
                // Check if the value is being moved
                if let Some((moved_var, _)) = self.check_move(value) {
                    self.mark_moved(&moved_var, span.clone());
                } else {
                    self.check_node(value);
                }
                // New variable takes ownership
                self.variables.insert(name.clone(), VarState::Owned);
            }
            AstNode::SetStmt { target, value, span } => {
                // Check the target - only check simple identifiers
                if let AstNode::Ident { name, .. } = target.as_ref() {
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
                }
                // Check the target and value expressions
                self.check_node(target);
                self.check_node(value);
            }
            AstNode::Ident { name, span } => {
                // Check if variable is moved
                if let Some(VarState::Moved(moved_at)) = self.variables.get(name) {
                    self.errors.push(BorrowError::UseAfterMove {
                        variable: name.clone(),
                        moved_at: moved_at.clone(),
                        used_at: span.clone(),
                    });
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
            AstNode::Call { callee, type_args: _, args, span: _ } => {
                self.check_node(callee);
                // For each argument, check if it's being moved
                for arg in args {
                    match arg {
                        // Explicit borrow - no move
                        AstNode::BorrowExpr { value, mutable, span: borrow_span } => {
                            // Extract the variable being borrowed
                            if let AstNode::Ident { name, .. } = value.as_ref() {
                                // Check if variable can be borrowed
                                if let Some(state) = self.variables.get(name) {
                                    match state {
                                        VarState::Moved(moved_at) => {
                                            self.errors.push(BorrowError::BorrowOfMovedValue {
                                                variable: name.clone(),
                                                moved_at: moved_at.clone(),
                                                borrowed_at: borrow_span.clone(),
                                            });
                                        }
                                        VarState::ImmutablyBorrowed(_) if *mutable => {
                                            // Trying to mutably borrow while immutably borrowed
                                            // This will be caught elsewhere
                                        }
                                        VarState::MutablyBorrowed(_) => {
                                            // Trying to borrow while mutably borrowed
                                            // This will be caught elsewhere
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            self.check_node(value);
                        }
                        // Direct identifier - potential move
                        // (Unless the function parameter is explicitly borrowed, but we don't track that yet)
                        _ => {
                            if let Some((moved_var, var_span)) = self.check_move(arg) {
                                // Conservative: assume non-borrowed args might move
                                // TODO: Check function signature to see if parameter is actually owned
                                self.mark_moved(&moved_var, var_span);
                            } else {
                                self.check_node(arg);
                            }
                        }
                    }
                }
            }
            AstNode::YieldStmt { value, span } => {
                // Yielding might move the value
                if let Some((moved_var, _)) = self.check_move(value) {
                    self.mark_moved(&moved_var, span.clone());
                } else {
                    self.check_node(value);
                }
            }
            AstNode::BorrowExpr { value, .. } => {
                // Borrow expression - just check the inner value
                // The borrow itself doesn't move
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

    #[test]
    fn test_move_tracking_basic() {
        let mut checker = BorrowChecker::new();

        // bind data to [1, 2, 3]
        // bind moved to data   # data is moved here
        // use data again        # ERROR: use after move
        let nodes = vec![
            AstNode::BindStmt {
                name: "data".to_string(),
                typ: None,
                value: Box::new(AstNode::List {
                    elements: vec![],
                    span: SourceSpan::unknown(),
                }),
                span: SourceSpan::unknown(),
            },
            AstNode::BindStmt {
                name: "moved".to_string(),
                typ: None,
                value: Box::new(AstNode::Ident {
                    name: "data".to_string(),
                    span: SourceSpan::unknown(),
                }),
                span: SourceSpan::unknown(),
            },
            // Try to use data after it was moved
            AstNode::Ident {
                name: "data".to_string(),
                span: SourceSpan::unknown(),
            },
        ];

        let result = checker.check(&nodes);
        assert!(result.is_err(), "Should detect use-after-move");

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);

        match &errors[0] {
            BorrowError::UseAfterMove { variable, .. } => {
                assert_eq!(variable, "data");
            }
            _ => panic!("Expected UseAfterMove error"),
        }
    }

    #[test]
    fn test_borrow_prevents_move() {
        let mut checker = BorrowChecker::new();

        // bind data to [1, 2, 3]
        // process(borrow data)  # borrow, not move
        // use data again         # OK: data still owned
        let nodes = vec![
            AstNode::BindStmt {
                name: "data".to_string(),
                typ: None,
                value: Box::new(AstNode::List {
                    elements: vec![],
                    span: SourceSpan::unknown(),
                }),
                span: SourceSpan::unknown(),
            },
            AstNode::Call {
                callee: Box::new(AstNode::Ident {
                    name: "process".to_string(),
                    span: SourceSpan::unknown(),
                }),
                type_args: vec![],
                args: vec![AstNode::BorrowExpr {
                    value: Box::new(AstNode::Ident {
                        name: "data".to_string(),
                        span: SourceSpan::unknown(),
                    }),
                    mutable: false,
                    span: SourceSpan::unknown(),
                }],
                span: SourceSpan::unknown(),
            },
            // Try to use data after borrowing it
            AstNode::Ident {
                name: "data".to_string(),
                span: SourceSpan::unknown(),
            },
        ];

        let result = checker.check(&nodes);
        // Should be OK - borrowing doesn't move
        assert!(result.is_ok(), "Borrowing should not move the value");
    }
}
