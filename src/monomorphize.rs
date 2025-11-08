//! # Monomorphization Module
//!
//! Transforms generic functions into specialized (monomorphized) versions
//! for each unique type instantiation.
//!
//! ## Example
//!
//! Input:
//! ```glimmer
//! chant identity<T>(x: T) -> T then
//!     yield x
//! end
//!
//! identity<Number>(42)
//! identity<Text>("hello")
//! ```
//!
//! Output (conceptual):
//! ```glimmer
//! chant identity_Number(x: Number) -> Number then
//!     yield x
//! end
//!
//! chant identity_Text(x: Text) -> Text then
//!     yield x
//! end
//!
//! identity_Number(42)
//! identity_Text("hello")
//! ```

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use crate::ast::*;

/// Represents a type instantiation of a generic function
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TypeInstantiation {
    /// Name of the generic function
    function_name: String,
    /// Type arguments for this instantiation
    type_args: Vec<String>,
}

impl TypeInstantiation {
    /// Generate a specialized function name
    /// Example: (identity, [Number]) → "identity_Number"
    fn specialized_name(&self) -> String {
        if self.type_args.is_empty() {
            self.function_name.clone()
        } else {
            format!("{}_{}", self.function_name, self.type_args.join("_"))
        }
    }
}

/// Monomorphizer transforms generic code into specialized versions
pub struct Monomorphizer {
    /// Maps (function_name, type_args) to specialized function name
    instantiations: BTreeMap<TypeInstantiation, String>,
    /// Original generic function definitions
    generic_functions: BTreeMap<String, AstNode>,
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Monomorphizer {
    pub fn new() -> Self {
        Monomorphizer {
            instantiations: BTreeMap::new(),
            generic_functions: BTreeMap::new(),
        }
    }

    /// Monomorphize an AST
    /// Returns a new AST with generic functions replaced by specialized versions
    pub fn monomorphize(&mut self, nodes: &[AstNode]) -> Vec<AstNode> {
        // Phase 1: Collect generic function definitions
        self.collect_generic_functions(nodes);

        // Phase 2: Find all instantiations (calls with type arguments)
        self.find_instantiations(nodes);

        // Phase 3: Generate specialized functions
        let specialized_functions = self.generate_specialized_functions();

        // Phase 4: Transform the AST - replace calls and remove generic functions
        let mut result = Vec::new();

        // Add specialized functions
        result.extend(specialized_functions);

        // Add transformed non-generic nodes
        for node in nodes {
            if !self.is_generic_function(node) {
                result.push(self.transform_node(node));
            }
        }

        result
    }

    /// Collect all generic function definitions
    fn collect_generic_functions(&mut self, nodes: &[AstNode]) {
        for node in nodes {
            if let AstNode::ChantDef { name, type_params, .. } = node {
                if !type_params.is_empty() {
                    self.generic_functions.insert(name.clone(), node.clone());
                }
            }
        }
    }

    /// Find all instantiations by scanning for calls with type arguments
    fn find_instantiations(&mut self, nodes: &[AstNode]) {
        for node in nodes {
            self.find_instantiations_in_node(node);
        }
    }

    /// Recursively find instantiations in a node
    fn find_instantiations_in_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Call { callee, type_args, args, .. } => {
                // If this is a call with type arguments to a generic function
                if !type_args.is_empty() {
                    if let AstNode::Ident { name: func_name, .. } = &**callee {
                        if self.generic_functions.contains_key(func_name) {
                            let type_arg_names: Vec<String> = type_args
                                .iter()
                                .map(|ta| self.type_annotation_to_string(ta))
                                .collect();

                            let instantiation = TypeInstantiation {
                                function_name: func_name.clone(),
                                type_args: type_arg_names,
                            };

                            let specialized = instantiation.specialized_name();
                            self.instantiations.insert(instantiation, specialized);
                        }
                    }
                }

                // Recurse into arguments
                for arg in args {
                    self.find_instantiations_in_node(arg);
                }
            }

            // Recurse into other node types
            AstNode::BinaryOp { left, right, .. } => {
                self.find_instantiations_in_node(left);
                self.find_instantiations_in_node(right);
            }

            AstNode::UnaryOp { operand, .. } => {
                self.find_instantiations_in_node(operand);
            }

            AstNode::IfStmt { condition, then_branch, else_branch, .. } => {
                self.find_instantiations_in_node(condition);
                for stmt in then_branch {
                    self.find_instantiations_in_node(stmt);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.find_instantiations_in_node(stmt);
                    }
                }
            }

            AstNode::WhileStmt { condition, body, .. } |
            AstNode::ForStmt { iterable: condition, body, .. } => {
                self.find_instantiations_in_node(condition);
                for stmt in body {
                    self.find_instantiations_in_node(stmt);
                }
            }

            AstNode::BindStmt { value, .. } |
            AstNode::WeaveStmt { value, .. } |
            AstNode::SetStmt { value, .. } |
            AstNode::YieldStmt { value, .. } => {
                self.find_instantiations_in_node(value);
            }

            AstNode::ExprStmt { expr, .. } => {
                self.find_instantiations_in_node(expr);
            }

            AstNode::List { elements, .. } => {
                for elem in elements {
                    self.find_instantiations_in_node(elem);
                }
            }

            AstNode::ChantDef { body, .. } => {
                for stmt in body {
                    self.find_instantiations_in_node(stmt);
                }
            }

            AstNode::Block { statements, .. } => {
                for stmt in statements {
                    self.find_instantiations_in_node(stmt);
                }
            }

            AstNode::Try { expr, .. } => {
                self.find_instantiations_in_node(expr);
            }

            // Other nodes don't contain calls
            _ => {}
        }
    }

    /// Convert TypeAnnotation to String for instantiation tracking
    fn type_annotation_to_string(&self, ann: &TypeAnnotation) -> String {
        monomorphize_type_annotation_to_string(ann)
    }
}

/// Convert TypeAnnotation to String for instantiation tracking (standalone helper)
fn monomorphize_type_annotation_to_string(ann: &TypeAnnotation) -> String {
    match ann {
        TypeAnnotation::Named(name) => name.clone(),
        TypeAnnotation::Generic(name) => name.clone(),
        TypeAnnotation::List(inner) => {
            format!("List_{}", monomorphize_type_annotation_to_string(inner))
        }
        TypeAnnotation::Parametrized { name, type_args } => {
            let args: Vec<String> = type_args
                .iter()
                .map(monomorphize_type_annotation_to_string)
                .collect();
            format!("{}_{}", name, args.join("_"))
        }
        TypeAnnotation::Map => "Map".to_string(),
        TypeAnnotation::Function { .. } => "Function".to_string(),
        TypeAnnotation::Optional(inner) => {
            format!("Optional_{}", monomorphize_type_annotation_to_string(inner))
        }
        TypeAnnotation::Borrowed { lifetime, inner, mutable } => {
            let lifetime_str = lifetime
                .as_ref()
                .map(|lt| format!("_{}", lt.name))
                .unwrap_or_default();
            let mut_str = if *mutable { "_mut" } else { "" };
            format!(
                "Borrowed{}{}_{}",
                lifetime_str,
                mut_str,
                monomorphize_type_annotation_to_string(inner)
            )
        }
    }
}

impl Monomorphizer {

    /// Generate specialized function definitions
    fn generate_specialized_functions(&self) -> Vec<AstNode> {
        let mut specialized = Vec::new();

        for (instantiation, specialized_name) in &self.instantiations {
            if let Some(generic_def) = self.generic_functions.get(&instantiation.function_name) {
                let specialized_func = self.specialize_function(
                    generic_def,
                    &instantiation.type_args,
                    specialized_name,
                );
                specialized.push(specialized_func);
            }
        }

        specialized
    }

    /// Specialize a generic function for specific type arguments
    fn specialize_function(
        &self,
        generic_def: &AstNode,
        type_args: &[String],
        specialized_name: &str,
    ) -> AstNode {
        if let AstNode::ChantDef {
            name: _,
            type_params,
            lifetime_params,
            params,
            return_type,
            body,
            span,
        } = generic_def
        {
            // Build substitution map: type parameter -> concrete type
            let mut substitutions = BTreeMap::new();
            for (param, arg) in type_params.iter().zip(type_args.iter()) {
                substitutions.insert(param.clone(), arg.clone());
            }

            // Substitute type annotations in parameters
            let specialized_params: Vec<Parameter> = params
                .iter()
                .map(|p| Parameter {
                    name: p.name.clone(),
                    typ: p.typ.as_ref().map(|t| self.substitute_type_annotation(t, &substitutions)),
                    is_variadic: p.is_variadic,
                    borrow_mode: p.borrow_mode.clone(),
                    lifetime: p.lifetime.clone(),
                })
                .collect();

            // Substitute return type
            let specialized_return = return_type
                .as_ref()
                .map(|t| self.substitute_type_annotation(t, &substitutions));

            // Create specialized function (no type parameters)
            AstNode::ChantDef {
                name: specialized_name.to_string(),
                type_params: vec![], // No type parameters in specialized version
                lifetime_params: lifetime_params.clone(),
                params: specialized_params,
                return_type: specialized_return,
                body: body.clone(), // Body doesn't need type substitution
                span: span.clone(),
            }
        } else {
            panic!("Expected ChantDef");
        }
    }

    /// Substitute type parameters in a type annotation
    fn substitute_type_annotation(
        &self,
        ann: &TypeAnnotation,
        substitutions: &BTreeMap<String, String>,
    ) -> TypeAnnotation {
        substitute_type_annotation_helper(ann, substitutions)
    }
}

/// Substitute type parameters in a type annotation (standalone helper)
fn substitute_type_annotation_helper(
    ann: &TypeAnnotation,
    substitutions: &BTreeMap<String, String>,
) -> TypeAnnotation {
    match ann {
        TypeAnnotation::Generic(name) => {
            // Replace type parameter with concrete type
            if let Some(concrete) = substitutions.get(name) {
                TypeAnnotation::Named(concrete.clone())
            } else {
                ann.clone()
            }
        }
        TypeAnnotation::List(inner) => {
            TypeAnnotation::List(Box::new(substitute_type_annotation_helper(inner, substitutions)))
        }
        TypeAnnotation::Parametrized { name, type_args } => {
            TypeAnnotation::Parametrized {
                name: name.clone(),
                type_args: type_args
                    .iter()
                    .map(|arg| substitute_type_annotation_helper(arg, substitutions))
                    .collect(),
            }
        }
        TypeAnnotation::Function { param_types, return_type } => {
            TypeAnnotation::Function {
                param_types: param_types
                    .iter()
                    .map(|pt| substitute_type_annotation_helper(pt, substitutions))
                    .collect(),
                return_type: Box::new(substitute_type_annotation_helper(return_type, substitutions)),
            }
        }
        TypeAnnotation::Optional(inner) => {
            TypeAnnotation::Optional(Box::new(substitute_type_annotation_helper(inner, substitutions)))
        }
        TypeAnnotation::Borrowed { lifetime, inner, mutable } => {
            TypeAnnotation::Borrowed {
                lifetime: lifetime.clone(),
                inner: Box::new(substitute_type_annotation_helper(inner, substitutions)),
                mutable: *mutable,
            }
        }
        // Named, Map don't need substitution
        _ => ann.clone(),
    }
}

impl Monomorphizer {

    /// Transform a node, replacing generic calls with calls to specialized versions
    fn transform_node(&self, node: &AstNode) -> AstNode {
        match node {
            AstNode::Call { callee, type_args, args, span } => {
                // Check if this is a call to a generic function
                if !type_args.is_empty() {
                    if let AstNode::Ident { name: func_name, .. } = &**callee {
                        let type_arg_names: Vec<String> = type_args
                            .iter()
                            .map(|ta| self.type_annotation_to_string(ta))
                            .collect();

                        let instantiation = TypeInstantiation {
                            function_name: func_name.clone(),
                            type_args: type_arg_names,
                        };

                        if let Some(specialized_name) = self.instantiations.get(&instantiation) {
                            // Replace with call to specialized function
                            return AstNode::Call {
                                callee: Box::new(AstNode::Ident {
                                    name: specialized_name.clone(),
                                    span: span.clone(),
                                }),
                                type_args: vec![], // No type args in specialized call
                                args: args.iter().map(|arg| self.transform_node(arg)).collect(),
                                span: span.clone(),
                            };
                        }
                    }
                }

                // Not a generic call, just transform arguments
                AstNode::Call {
                    callee: Box::new(self.transform_node(callee)),
                    type_args: type_args.clone(),
                    args: args.iter().map(|arg| self.transform_node(arg)).collect(),
                    span: span.clone(),
                }
            }

            // Transform other nodes recursively
            AstNode::BinaryOp { left, op, right, span } => AstNode::BinaryOp {
                left: Box::new(self.transform_node(left)),
                op: *op,
                right: Box::new(self.transform_node(right)),
                span: span.clone(),
            },

            AstNode::UnaryOp { op, operand, span } => AstNode::UnaryOp {
                op: *op,
                operand: Box::new(self.transform_node(operand)),
                span: span.clone(),
            },

            AstNode::BindStmt { name, typ, value, span } => AstNode::BindStmt {
                name: name.clone(),
                typ: typ.clone(),
                value: Box::new(self.transform_node(value)),
                span: span.clone(),
            },

            AstNode::WeaveStmt { name, typ, value, span } => AstNode::WeaveStmt {
                name: name.clone(),
                typ: typ.clone(),
                value: Box::new(self.transform_node(value)),
                span: span.clone(),
            },

            AstNode::SetStmt { target, value, span } => AstNode::SetStmt {
                target: Box::new(self.transform_node(target)),
                value: Box::new(self.transform_node(value)),
                span: span.clone(),
            },

            AstNode::YieldStmt { value, span } => AstNode::YieldStmt {
                value: Box::new(self.transform_node(value)),
                span: span.clone(),
            },

            AstNode::ExprStmt { expr, span } => AstNode::ExprStmt {
                expr: Box::new(self.transform_node(expr)),
                span: span.clone(),
            },

            AstNode::List { elements, span } => AstNode::List {
                elements: elements.iter().map(|elem| self.transform_node(elem)).collect(),
                span: span.clone(),
            },

            AstNode::Block { statements, span } => AstNode::Block {
                statements: statements.iter().map(|stmt| self.transform_node(stmt)).collect(),
                span: span.clone(),
            },

            AstNode::Try { expr, span } => AstNode::Try {
                expr: Box::new(self.transform_node(expr)),
                span: span.clone(),
            },

            // ChantDef is not transformed here (handled separately)
            // All other nodes are unchanged
            _ => node.clone(),
        }
    }

    /// Check if a node is a generic function definition
    fn is_generic_function(&self, node: &AstNode) -> bool {
        if let AstNode::ChantDef { name, type_params, .. } = node {
            !type_params.is_empty() && self.generic_functions.contains_key(name)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_instantiation_specialized_name() {
        let inst = TypeInstantiation {
            function_name: "identity".to_string(),
            type_args: vec!["Number".to_string()],
        };
        assert_eq!(inst.specialized_name(), "identity_Number");

        let inst2 = TypeInstantiation {
            function_name: "pair".to_string(),
            type_args: vec!["Number".to_string(), "Text".to_string()],
        };
        assert_eq!(inst2.specialized_name(), "pair_Number_Text");
    }

    #[test]
    fn test_monomorphize_simple_identity() {
        use crate::source_location::SourceSpan;
        let dummy_span = SourceSpan::default();

        let ast = vec![
            AstNode::ChantDef {
                name: "identity".to_string(),
                type_params: vec!["T".to_string()],
                lifetime_params: vec![],
                params: vec![Parameter {
                    name: "x".to_string(),
                    typ: Some(TypeAnnotation::Generic("T".to_string())),
                    is_variadic: false,
                    borrow_mode: BorrowMode::Owned,
                    lifetime: None,
                }],
                return_type: Some(TypeAnnotation::Generic("T".to_string())),
                body: vec![AstNode::YieldStmt {
                    value: Box::new(AstNode::Ident {
                        name: "x".to_string(),
                        span: dummy_span.clone(),
                    }),
                    span: dummy_span.clone(),
                }],
                span: dummy_span.clone(),
            },
            AstNode::ExprStmt {
                expr: Box::new(AstNode::Call {
                    callee: Box::new(AstNode::Ident {
                        name: "identity".to_string(),
                        span: dummy_span.clone(),
                    }),
                    type_args: vec![TypeAnnotation::Named("Number".to_string())],
                    args: vec![AstNode::Number {
                        value: 42.0,
                        span: dummy_span.clone(),
                    }],
                    span: dummy_span.clone(),
                }),
                span: dummy_span.clone(),
            },
        ];

        let mut mono = Monomorphizer::new();
        let result = mono.monomorphize(&ast);

        // Should have specialized function + transformed call
        assert_eq!(result.len(), 2);

        // First should be specialized function
        if let AstNode::ChantDef { name, type_params, params, .. } = &result[0] {
            assert_eq!(name, "identity_Number");
            assert!(type_params.is_empty());
            assert_eq!(params[0].typ, Some(TypeAnnotation::Named("Number".to_string())));
        } else {
            panic!("Expected specialized ChantDef");
        }

        // Second should be transformed call
        if let AstNode::ExprStmt { expr, .. } = &result[1] {
            if let AstNode::Call { callee, type_args, .. } = &**expr {
                if let AstNode::Ident { name, .. } = &**callee {
                    assert_eq!(name, "identity_Number");
                    assert!(type_args.is_empty());
                } else {
                    panic!("Expected Ident callee");
                }
            } else {
                panic!("Expected Call");
            }
        } else {
            panic!("Expected ExprStmt");
        }
    }
}
