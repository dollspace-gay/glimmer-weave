//! Symbol table for tracking definitions and their locations
//!
//! This module provides symbol table functionality for the LSP server,
//! enabling features like go-to-definition and find-references.

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{AstNode, Parameter};
use crate::source_location::SourceSpan;

/// Type of symbol in the symbol table
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// Immutable variable (bind)
    Variable,
    /// Mutable variable (weave)
    MutableVariable,
    /// Function definition (chant)
    Function,
    /// Struct definition (form)
    Struct,
    /// Function parameter
    Parameter,
}

/// A symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Location where the symbol is defined
    pub definition_span: SourceSpan,
    /// For functions: parameter names and types
    pub parameters: Vec<Parameter>,
    /// For functions: return type annotation
    pub return_type: Option<String>,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: String, kind: SymbolKind, definition_span: SourceSpan) -> Self {
        Self {
            name,
            kind,
            definition_span,
            parameters: Vec::new(),
            return_type: None,
        }
    }

    /// Create a function symbol with parameters
    pub fn function(
        name: String,
        definition_span: SourceSpan,
        parameters: Vec<Parameter>,
        return_type: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: SymbolKind::Function,
            definition_span,
            parameters,
            return_type,
        }
    }
}

/// Symbol table for tracking all symbols in a document
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// Map from symbol name to symbol information
    symbols: HashMap<String, Vec<Symbol>>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Add a symbol to the table
    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols
            .entry(symbol.name.clone())
            .or_default()
            .push(symbol);
    }

    /// Look up all symbols with a given name
    pub fn lookup(&self, name: &str) -> Option<&[Symbol]> {
        self.symbols.get(name).map(|v| v.as_slice())
    }

    /// Find the symbol at or before a given position
    ///
    /// This is used for go-to-definition: given a cursor position,
    /// find the symbol that was most recently defined before that position.
    pub fn find_at_position(&self, name: &str, line: usize, column: usize) -> Option<&Symbol> {
        self.lookup(name)?
            .iter()
            .filter(|sym| {
                // Symbol must be defined before the position
                if sym.definition_span.start.is_known() {
                    sym.definition_span.start.line < line
                        || (sym.definition_span.start.line == line
                            && sym.definition_span.start.column <= column)
                } else {
                    false
                }
            })
            .max_by_key(|sym| {
                if sym.definition_span.start.is_known() {
                    (sym.definition_span.start.line, sym.definition_span.start.column)
                } else {
                    (0, 0)
                }
            })
    }

    /// Get all symbols in the table
    pub fn all_symbols(&self) -> Vec<&Symbol> {
        self.symbols
            .values()
            .flat_map(|symbols| symbols.iter())
            .collect()
    }

    /// Get all symbols of a specific kind
    pub fn symbols_of_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.all_symbols()
            .into_iter()
            .filter(|sym| sym.kind == kind)
            .collect()
    }
}

/// Collector for building a symbol table from an AST
pub struct SymbolCollector {
    table: SymbolTable,
}

impl SymbolCollector {
    /// Create a new symbol collector
    pub fn new() -> Self {
        Self {
            table: SymbolTable::new(),
        }
    }

    /// Collect symbols from an AST and return the symbol table
    pub fn collect(mut self, nodes: &[AstNode]) -> SymbolTable {
        for node in nodes {
            self.visit_node(node);
        }
        self.table
    }

    /// Visit an AST node and extract symbols
    fn visit_node(&mut self, node: &AstNode) {
        match node {
            // Immutable variable binding
            AstNode::BindStmt {
                name, span, value, ..
            } => {
                self.table.insert(Symbol::new(
                    name.clone(),
                    SymbolKind::Variable,
                    span.clone(),
                ));
                self.visit_node(value);
            }

            // Mutable variable
            AstNode::WeaveStmt {
                name, span, value, ..
            } => {
                self.table.insert(Symbol::new(
                    name.clone(),
                    SymbolKind::MutableVariable,
                    span.clone(),
                ));
                self.visit_node(value);
            }

            // Function definition
            AstNode::ChantDef {
                name,
                params,
                body,
                return_type,
                span,
                ..
            } => {
                let return_type_str = return_type.as_ref().map(|t| format!("{:?}", t));
                self.table.insert(Symbol::function(
                    name.clone(),
                    span.clone(),
                    params.clone(),
                    return_type_str,
                ));

                // Visit function body
                for stmt in body {
                    self.visit_node(stmt);
                }
            }

            // Struct definition
            AstNode::FormDef { name, span, .. } => {
                self.table.insert(Symbol::new(
                    name.clone(),
                    SymbolKind::Struct,
                    span.clone(),
                ));
            }

            // Recursively visit other node types
            AstNode::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit_node(condition);
                for stmt in then_branch {
                    self.visit_node(stmt);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.visit_node(stmt);
                    }
                }
            }

            AstNode::WhileStmt { condition, body, .. } => {
                self.visit_node(condition);
                for stmt in body {
                    self.visit_node(stmt);
                }
            }

            AstNode::ForStmt { body, iterable, .. } => {
                self.visit_node(iterable);
                for stmt in body {
                    self.visit_node(stmt);
                }
            }

            AstNode::Block { statements, .. } => {
                for stmt in statements {
                    self.visit_node(stmt);
                }
            }

            AstNode::BinaryOp { left, right, .. } => {
                self.visit_node(left);
                self.visit_node(right);
            }

            AstNode::UnaryOp { operand, .. } => {
                self.visit_node(operand);
            }

            AstNode::Call { callee, args, .. } => {
                self.visit_node(callee);
                for arg in args {
                    self.visit_node(arg);
                }
            }

            AstNode::List { elements, .. } => {
                for elem in elements {
                    self.visit_node(elem);
                }
            }

            AstNode::Map { entries, .. } => {
                for (_, value) in entries {
                    self.visit_node(value);
                }
            }

            AstNode::IndexAccess { object, index, .. } => {
                self.visit_node(object);
                self.visit_node(index);
            }

            AstNode::FieldAccess { object, .. } => {
                self.visit_node(object);
            }

            AstNode::SetStmt { target, value, .. } => {
                self.visit_node(target);
                self.visit_node(value);
            }

            AstNode::YieldStmt { value, .. } => {
                self.visit_node(value);
            }

            AstNode::AttemptStmt {
                body,
                handlers,
                ..
            } => {
                for stmt in body {
                    self.visit_node(stmt);
                }
                for handler in handlers {
                    for stmt in &handler.body {
                        self.visit_node(stmt);
                    }
                }
            }

            AstNode::MatchStmt {
                value, arms, ..
            } => {
                self.visit_node(value);
                for arm in arms {
                    for stmt in &arm.body {
                        self.visit_node(stmt);
                    }
                }
            }

            AstNode::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.visit_node(value);
                }
            }

            AstNode::RequestStmt { capability, .. } => {
                self.visit_node(capability);
            }

            AstNode::ModuleDecl { body, .. } => {
                for stmt in body {
                    self.visit_node(stmt);
                }
            }

            // Leaf nodes - no children to visit
            AstNode::Number { .. }
            | AstNode::Text { .. }
            | AstNode::Truth { .. }
            | AstNode::Nothing { .. }
            | AstNode::Ident { .. }
            | AstNode::Range { .. }
            | AstNode::Triumph { .. }
            | AstNode::Mishap { .. }
            | AstNode::Present { .. }
            | AstNode::Absent { .. }
            | AstNode::Import { .. }
            | AstNode::Export { .. }
            | AstNode::ModuleAccess { .. }
            | AstNode::BorrowExpr { .. }
            | AstNode::Pipeline { .. }
            | AstNode::SeekExpr { .. }
            | AstNode::ExprStmt { .. }
            | AstNode::Break { .. }
            | AstNode::Continue { .. }
            | AstNode::Try { .. }
            | AstNode::VariantDef { .. }
            | AstNode::AspectDef { .. }
            | AstNode::EmbodyStmt { .. } => {
                // No children to visit
            }
        }
    }
}

impl Default for SymbolCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_location::{SourceLocation, SourceSpan};

    #[test]
    fn test_symbol_table_insert_and_lookup() {
        let mut table = SymbolTable::new();

        let sym = Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            SourceSpan::new(SourceLocation::new(1, 1), SourceLocation::new(1, 10)),
        );
        table.insert(sym);

        let found = table.lookup("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().len(), 1);
        assert_eq!(found.unwrap()[0].name, "x");
    }

    #[test]
    fn test_symbol_table_multiple_definitions() {
        let mut table = SymbolTable::new();

        // Same name at different positions (e.g., shadowing)
        table.insert(Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            SourceSpan::new(SourceLocation::new(1, 1), SourceLocation::new(1, 10)),
        ));
        table.insert(Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            SourceSpan::new(SourceLocation::new(10, 1), SourceLocation::new(10, 10)),
        ));

        let found = table.lookup("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().len(), 2);
    }

    #[test]
    fn test_find_at_position() {
        let mut table = SymbolTable::new();

        table.insert(Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            SourceSpan::new(SourceLocation::new(1, 5), SourceLocation::new(1, 10)),
        ));
        table.insert(Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            SourceSpan::new(SourceLocation::new(10, 5), SourceLocation::new(10, 10)),
        ));

        // At line 5, should find the first definition
        let sym = table.find_at_position("x", 5, 1);
        assert!(sym.is_some());
        assert_eq!(sym.unwrap().definition_span.start.line, 1);

        // At line 15, should find the second definition
        let sym = table.find_at_position("x", 15, 1);
        assert!(sym.is_some());
        assert_eq!(sym.unwrap().definition_span.start.line, 10);

        // Before any definition, should find nothing
        let sym = table.find_at_position("x", 1, 1);
        assert!(sym.is_none());
    }

    #[test]
    fn test_symbols_of_kind() {
        let mut table = SymbolTable::new();

        table.insert(Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            SourceSpan::new(SourceLocation::new(1, 1), SourceLocation::new(1, 10)),
        ));
        table.insert(Symbol::new(
            "add".to_string(),
            SymbolKind::Function,
            SourceSpan::new(SourceLocation::new(3, 1), SourceLocation::new(3, 10)),
        ));
        table.insert(Symbol::new(
            "Point".to_string(),
            SymbolKind::Struct,
            SourceSpan::new(SourceLocation::new(5, 1), SourceLocation::new(5, 10)),
        ));

        let funcs = table.symbols_of_kind(SymbolKind::Function);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "add");

        let structs = table.symbols_of_kind(SymbolKind::Struct);
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Point");
    }
}
