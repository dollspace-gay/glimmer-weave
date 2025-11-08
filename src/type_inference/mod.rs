/// Type Inference Module
///
/// Implements Hindley-Milner type inference with natural language branding.
///
/// ## Overview
///
/// This module provides automatic type inference for Glimmer-Weave, allowing
/// developers to write code without explicit type annotations while maintaining
/// full type safety.
///
/// ## Key Components
///
/// - **TypeVar** - Type variables (α, β, γ) representing unknown types
/// - **InferType** - Extended type representation for inference
/// - **Requirement** - Type equations that must be satisfied
/// - **Harmonizer** - Unification algorithm (harmonize types)
/// - **ConstraintGenerator** - Generate type requirements from AST
///
/// ## Natural Language Naming
///
/// Following Glimmer-Weave's philosophy:
/// - `harmonize` - Unify types (find common type)
/// - `materialize` - Apply substitutions (make concrete)
/// - `abstract` - Generalize (create polymorphic types)
/// - `specialize` - Instantiate (create fresh type variables)
///
/// ## Usage
///
/// ```rust,no_run
/// use glimmer_weave::type_inference::TypeInference;
/// use glimmer_weave::ast::AstNode;
/// use glimmer_weave::type_inference::TypeError;
///
/// fn example() -> Result<(), TypeError> {
///     let mut inference = TypeInference::new();
///     let ast = vec![AstNode::Number(42.0)];
///     let typed_ast = inference.infer_types(&ast)?;
///     Ok(())
/// }
/// ```
pub mod type_var;
pub mod infer_type;
pub mod requirement;
pub mod harmonize;
pub mod constraints;
pub mod scheme;
pub mod errors;

pub use type_var::TypeVar;
pub use infer_type::InferType;
pub use requirement::{Requirement, RequirementSet};
pub use harmonize::Harmonizer;
pub use constraints::ConstraintGenerator;
pub use scheme::TypeScheme;
pub use errors::TypeError;

use crate::ast::AstNode;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

/// Main type inference engine
///
/// Coordinates all phases of type inference:
/// 1. Constraint generation from AST
/// 2. Harmonization (unification) to solve constraints
/// 3. Abstraction (generalization) for let-polymorphism
/// 4. Materialize final types
///
/// FUTURE: The `env` field will be used when type inference is integrated
/// into the main evaluation pipeline, enabling full Hindley-Milner type
/// inference with let-polymorphism and type variable resolution.
#[allow(dead_code)]
pub struct TypeInference {
    /// Next available type variable ID
    next_var: usize,

    /// Type environment (variable → type)
    env: TypeEnvironment,
}

/// Type environment mapping variables to type schemes
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    bindings: BTreeMap<String, TypeScheme>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        TypeEnvironment {
            bindings: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, scheme: TypeScheme) {
        self.bindings.insert(name, scheme);
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }

    pub fn remove(&mut self, name: &str) {
        self.bindings.remove(name);
    }

    /// Get all free type variables in the environment
    pub fn free_vars(&self) -> BTreeSet<TypeVar> {
        let mut vars = BTreeSet::new();
        for scheme in self.bindings.values() {
            vars.extend(scheme.free_vars());
        }
        vars
    }
}

impl TypeInference {
    pub fn new() -> Self {
        TypeInference {
            next_var: 0,
            env: TypeEnvironment::new(),
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar::fresh(self.next_var);
        self.next_var += 1;
        var
    }

    /// Main entry point for type inference
    ///
    /// Takes an AST and returns a typed AST with all types inferred.
    pub fn infer_types(&mut self, _nodes: &[AstNode]) -> Result<Vec<AstNode>, TypeError> {
        // TODO: Implement full inference pipeline
        // 1. Generate constraints
        // 2. Harmonize (unify)
        // 3. Abstract (generalize)
        // 4. Materialize final types
        todo!("Full type inference pipeline")
    }

    /// Infer types for a program (simplified interface returning String errors)
    ///
    /// This is a simpler interface used by SemanticAnalyzer that returns
    /// String errors instead of the full TypeError type.
    ///
    /// # Algorithm
    ///
    /// 1. **Constraint Generation**: Walk AST and generate type constraints
    /// 2. **Harmonization (Unification)**: Solve constraints using Robinson's algorithm
    /// 3. **Type Resolution**: Apply substitutions to get final types
    ///
    /// # Arguments
    ///
    /// * `nodes` - AST nodes to infer types for
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Type inference succeeded
    /// * `Err(String)` - Type error with natural language message
    pub fn infer_program(&mut self, nodes: &[AstNode]) -> Result<(), String> {
        use alloc::collections::BTreeMap;
        use crate::semantic::Type;

        // Type variables and constraints
        let mut constraints: Vec<(Type, Type)> = Vec::new();
        let mut environment: BTreeMap<String, Type> = BTreeMap::new();
        let mut substitutions: BTreeMap<String, Type> = BTreeMap::new();

        // Step 1: Generate constraints from AST
        for node in nodes {
            self.generate_constraints_internal(
                node,
                &mut constraints,
                &mut environment,
            )?;
        }

        // Step 2: Solve constraints via unification
        for (ty1, ty2) in constraints {
            self.unify_internal(ty1, ty2, &mut substitutions)?;
        }

        // Step 3: Store inferred types in environment
        for (name, ty) in environment {
            let final_ty = self.apply_substitution_internal(&ty, &substitutions);
            self.env.insert(name, TypeScheme::mono(InferType::Concrete(final_ty)));
        }

        Ok(())
    }

    /// Generate type constraints from an AST node (internal helper)
    fn generate_constraints_internal(
        &mut self,
        node: &AstNode,
        constraints: &mut Vec<(crate::semantic::Type, crate::semantic::Type)>,
        environment: &mut BTreeMap<String, crate::semantic::Type>,
    ) -> Result<crate::semantic::Type, String> {
        use crate::ast::*;
        use crate::semantic::Type;

        match node {
            // Literals have known types
            AstNode::Number { .. } => Ok(Type::Number),
            AstNode::Text { .. } => Ok(Type::Text),
            AstNode::Truth { .. } => Ok(Type::Truth),
            AstNode::Nothing { .. } => Ok(Type::Nothing),

            // Variables lookup from environment
            AstNode::Ident { name, .. } => {
                if let Some(ty) = environment.get(name) {
                    Ok(ty.clone())
                } else {
                    let tv_id = self.fresh_var();
                    let tv = Type::TypeParam(tv_id.name().to_string());
                    environment.insert(name.clone(), tv.clone());
                    Ok(tv)
                }
            }

            // Binary operations
            AstNode::BinaryOp { left, op, right, .. } => {
                let left_ty = self.generate_constraints_internal(left, constraints, environment)?;
                let right_ty = self.generate_constraints_internal(right, constraints, environment)?;

                match op {
                    BinaryOperator::Add | BinaryOperator::Sub |
                    BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Mod => {
                        constraints.push((left_ty, Type::Number));
                        constraints.push((right_ty, Type::Number));
                        Ok(Type::Number)
                    }
                    BinaryOperator::Less | BinaryOperator::LessEq |
                    BinaryOperator::Greater | BinaryOperator::GreaterEq => {
                        constraints.push((left_ty, Type::Number));
                        constraints.push((right_ty, Type::Number));
                        Ok(Type::Truth)
                    }
                    BinaryOperator::Equal | BinaryOperator::NotEqual => {
                        constraints.push((left_ty.clone(), right_ty));
                        Ok(Type::Truth)
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        constraints.push((left_ty, Type::Truth));
                        constraints.push((right_ty, Type::Truth));
                        Ok(Type::Truth)
                    }
                }
            }

            // Unary operations
            AstNode::UnaryOp { op, operand, .. } => {
                let expr_ty = self.generate_constraints_internal(operand, constraints, environment)?;
                match op {
                    UnaryOperator::Negate => {
                        constraints.push((expr_ty, Type::Number));
                        Ok(Type::Number)
                    }
                    UnaryOperator::Not => {
                        constraints.push((expr_ty, Type::Truth));
                        Ok(Type::Truth)
                    }
                }
            }

            // Variable binding
            AstNode::BindStmt { name, value, typ: _, .. } => {
                let value_ty = self.generate_constraints_internal(value, constraints, environment)?;
                environment.insert(name.clone(), value_ty);
                Ok(Type::Nothing)
            }

            // Mutable variable
            AstNode::WeaveStmt { name, value, typ: _, .. } => {
                let value_ty = self.generate_constraints_internal(value, constraints, environment)?;
                environment.insert(name.clone(), value_ty);
                Ok(Type::Nothing)
            }

            // Assignment
            AstNode::SetStmt { name, value, .. } => {
                let value_ty = self.generate_constraints_internal(value, constraints, environment)?;
                if let Some(var_ty) = environment.get(name) {
                    constraints.push((var_ty.clone(), value_ty));
                } else {
                    return Err(format!("Undefined variable in assignment: {}", name));
                }
                Ok(Type::Nothing)
            }

            // Lists
            AstNode::List { elements, .. } => {
                if elements.is_empty() {
                    let tv_id = self.fresh_var();
                    Ok(Type::List(Box::new(Type::TypeParam(tv_id.name().to_string()))))
                } else {
                    let first_ty = self.generate_constraints_internal(&elements[0], constraints, environment)?;
                    for elem in &elements[1..] {
                        let elem_ty = self.generate_constraints_internal(elem, constraints, environment)?;
                        constraints.push((first_ty.clone(), elem_ty));
                    }
                    Ok(Type::List(Box::new(first_ty)))
                }
            }

            // Blocks
            AstNode::Block { statements, .. } => {
                let mut last_ty = Type::Nothing;
                for stmt in statements {
                    last_ty = self.generate_constraints_internal(stmt, constraints, environment)?;
                }
                Ok(last_ty)
            }

            // Conditionals
            AstNode::IfStmt { condition, then_branch, else_branch, .. } => {
                let cond_ty = self.generate_constraints_internal(condition, constraints, environment)?;
                constraints.push((cond_ty, Type::Truth));

                // then_branch is Vec<AstNode>, need to treat it as a block
                let then_ty = if then_branch.len() == 1 {
                    self.generate_constraints_internal(&then_branch[0], constraints, environment)?
                } else {
                    let mut last_ty = Type::Nothing;
                    for stmt in then_branch {
                        last_ty = self.generate_constraints_internal(stmt, constraints, environment)?;
                    }
                    last_ty
                };

                if let Some(else_stmts) = else_branch {
                    let else_ty = if else_stmts.len() == 1 {
                        self.generate_constraints_internal(&else_stmts[0], constraints, environment)?
                    } else {
                        let mut last_ty = Type::Nothing;
                        for stmt in else_stmts {
                            last_ty = self.generate_constraints_internal(stmt, constraints, environment)?;
                        }
                        last_ty
                    };
                    constraints.push((then_ty.clone(), else_ty));
                }

                Ok(then_ty)
            }

            // For other nodes, return Unknown for now
            _ => Ok(Type::Unknown),
        }
    }

    /// Unify two types (Robinson's unification algorithm)
    fn unify_internal(
        &self,
        ty1: crate::semantic::Type,
        ty2: crate::semantic::Type,
        substitutions: &mut BTreeMap<String, crate::semantic::Type>,
    ) -> Result<(), String> {
        use crate::semantic::Type;

        let t1 = self.apply_substitution_internal(&ty1, substitutions);
        let t2 = self.apply_substitution_internal(&ty2, substitutions);

        match (t1.clone(), t2.clone()) {
            // Identical types unify
            (a, b) if a == b => Ok(()),

            // Type variable unifies with any type
            (Type::TypeParam(var), ty) | (ty, Type::TypeParam(var)) => {
                if self.occurs_check_internal(&var, &ty) {
                    return Err(format!("Infinite type: {} occurs in {:?}", var, ty));
                }
                substitutions.insert(var, ty);
                Ok(())
            }

            // Unknown/Any unify with anything
            (Type::Unknown, _) | (_, Type::Unknown) => Ok(()),
            (Type::Any, _) | (_, Type::Any) => Ok(()),

            // List types: unify element types
            (Type::List(elem1), Type::List(elem2)) => {
                self.unify_internal(*elem1, *elem2, substitutions)
            }

            // Function types
            (Type::Function { params: params1, return_type: ret1 },
             Type::Function { params: params2, return_type: ret2 }) => {
                if params1.len() != params2.len() {
                    return Err(format!("Function arity mismatch: {} vs {}", params1.len(), params2.len()));
                }
                for (p1, p2) in params1.into_iter().zip(params2.into_iter()) {
                    self.unify_internal(p1, p2, substitutions)?;
                }
                self.unify_internal(*ret1, *ret2, substitutions)
            }

            // Incompatible types
            (t1, t2) => {
                Err(format!("Type mismatch: cannot unify {} and {}", t1.name(), t2.name()))
            }
        }
    }

    /// Check if a type variable occurs in a type (prevents infinite types)
    fn occurs_check_internal(&self, var: &str, ty: &crate::semantic::Type) -> bool {
        use crate::semantic::Type;

        match ty {
            Type::TypeParam(v) => v == var,
            Type::List(elem) => self.occurs_check_internal(var, elem),
            Type::Function { params, return_type } => {
                params.iter().any(|p| self.occurs_check_internal(var, p)) ||
                self.occurs_check_internal(var, return_type)
            }
            _ => false,
        }
    }

    /// Apply substitutions to a type
    fn apply_substitution_internal(
        &self,
        ty: &crate::semantic::Type,
        substitutions: &BTreeMap<String, crate::semantic::Type>,
    ) -> crate::semantic::Type {
        use crate::semantic::Type;

        match ty {
            Type::TypeParam(var) => {
                if let Some(substituted) = substitutions.get(var) {
                    self.apply_substitution_internal(substituted, substitutions)
                } else {
                    ty.clone()
                }
            }
            Type::List(elem) => {
                Type::List(Box::new(self.apply_substitution_internal(elem, substitutions)))
            }
            Type::Function { params, return_type } => {
                Type::Function {
                    params: params.iter().map(|p| self.apply_substitution_internal(p, substitutions)).collect(),
                    return_type: Box::new(self.apply_substitution_internal(return_type, substitutions)),
                }
            }
            _ => ty.clone(),
        }
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_var_generation() {
        let mut inference = TypeInference::new();
        let v1 = inference.fresh_var();
        let v2 = inference.fresh_var();
        let v3 = inference.fresh_var();

        assert_eq!(v1.id(), 0);
        assert_eq!(v2.id(), 1);
        assert_eq!(v3.id(), 2);

        assert_eq!(v1.name(), "α");
        assert_eq!(v2.name(), "β");
        assert_eq!(v3.name(), "γ");
    }

    #[test]
    fn test_type_environment_basic() {
        let mut env = TypeEnvironment::new();

        // Create simple scheme
        let scheme = TypeScheme::mono(InferType::Concrete(crate::semantic::Type::Number));
        env.insert("x".to_string(), scheme);

        assert!(env.lookup("x").is_some());
        assert!(env.lookup("y").is_none());

        env.remove("x");
        assert!(env.lookup("x").is_none());
    }
}
