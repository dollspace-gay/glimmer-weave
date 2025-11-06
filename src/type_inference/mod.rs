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
/// ```rust
/// use glimmer_weave::type_inference::{TypeInference, TypeVar, InferType};
///
/// let mut inference = TypeInference::new();
/// let typed_ast = inference.infer_types(&ast)?;
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
use std::collections::BTreeMap;

/// Main type inference engine
///
/// Coordinates all phases of type inference:
/// 1. Constraint generation from AST
/// 2. Harmonization (unification) to solve constraints
/// 3. Abstraction (generalization) for let-polymorphism
/// 4. Materialize final types
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
    pub fn free_vars(&self) -> std::collections::BTreeSet<TypeVar> {
        let mut vars = std::collections::BTreeSet::new();
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
