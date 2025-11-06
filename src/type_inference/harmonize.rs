/// Harmonization (Unification) Algorithm
///
/// Harmonization finds the most general type that satisfies two type expressions.
/// This is the core of Hindley-Milner type inference.
///
/// ## Natural Language Branding
///
/// - "harmonize" instead of "unify" - bringing types into harmony
/// - "materialize" instead of "apply substitutions" - making concrete
/// - "contains" instead of "occurs check" - does type contain variable?
///
/// ## Algorithm
///
/// ```text
/// harmonize(τ1, τ2, subst):
///   1. Materialize both types (apply current substitutions)
///   2. If identical → success
///   3. If one is variable:
///        - Check variable doesn't occur in other (infinite type)
///        - Add substitution: variable ↦ other
///   4. If both are constructors (e.g., List):
///        - Names must match
///        - Recursively harmonize arguments
///   5. Otherwise → type error
/// ```
///
/// ## Example
///
/// ```text
/// harmonize(α, Number) → {α ↦ Number}
/// harmonize(α → β, Number → Text) → {α ↦ Number, β ↦ Text}
/// harmonize(List<α>, List<Number>) → {α ↦ Number}
/// ```

use crate::type_inference::{InferType, TypeVar, TypeError};
use crate::type_inference::requirement::SourceLocation;
use std::collections::BTreeMap;

/// Harmonizer manages type unification and substitutions
pub struct Harmonizer {
    /// Current substitutions (type variable → type)
    substitutions: BTreeMap<TypeVar, InferType>,
}

impl Harmonizer {
    /// Create a new harmonizer
    pub fn new() -> Self {
        Harmonizer {
            substitutions: BTreeMap::new(),
        }
    }

    /// Harmonize two types (find most general unifier)
    ///
    /// This is the core unification algorithm. It attempts to find a
    /// substitution that makes both types equal.
    pub fn harmonize(&mut self, lhs: &InferType, rhs: &InferType) -> Result<(), TypeError> {
        self.harmonize_at(lhs, rhs, &SourceLocation::unknown())
    }

    /// Harmonize with source location for error reporting
    pub fn harmonize_at(
        &mut self,
        lhs: &InferType,
        rhs: &InferType,
        location: &SourceLocation,
    ) -> Result<(), TypeError> {
        // Materialize both types (apply current substitutions)
        let lhs = self.materialize(lhs);
        let rhs = self.materialize(rhs);

        match (&lhs, &rhs) {
            // Same type → success
            (l, r) if l == r => Ok(()),

            // Variable on left → bind it
            (InferType::Var(var), ty) | (ty, InferType::Var(var)) => {
                self.bind_var(var.clone(), ty.clone(), location)
            }

            // Function types → harmonize params and returns
            (InferType::Arrow(p1, r1), InferType::Arrow(p2, r2)) => {
                self.harmonize_at(p1, p2, location)?;
                self.harmonize_at(r1, r2, location)
            }

            // Generic types (List<T>, etc) → harmonize arguments
            (
                InferType::Generic { name: n1, args: a1 },
                InferType::Generic { name: n2, args: a2 },
            ) if n1 == n2 && a1.len() == a2.len() => {
                for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                    self.harmonize_at(arg1, arg2, location)?;
                }
                Ok(())
            }

            // Concrete types wrapped in Concrete
            (InferType::Concrete(t1), InferType::Concrete(t2)) if t1 == t2 => Ok(()),

            // Type mismatch
            _ => Err(TypeError::Mismatch {
                expected: lhs,
                got: rhs,
                location: location.clone(),
            }),
        }
    }

    /// Bind a type variable to a type
    fn bind_var(
        &mut self,
        var: TypeVar,
        ty: InferType,
        location: &SourceLocation,
    ) -> Result<(), TypeError> {
        // Occurs check: prevent infinite types
        if self.contains(&ty, &var) {
            return Err(TypeError::InfiniteType {
                var,
                ty,
                location: location.clone(),
            });
        }

        self.substitutions.insert(var, ty);
        Ok(())
    }

    /// Materialize a type (apply all substitutions)
    ///
    /// Recursively replaces type variables with their substitutions.
    pub fn materialize(&self, ty: &InferType) -> InferType {
        match ty {
            InferType::Var(var) => {
                if let Some(subst) = self.substitutions.get(var) {
                    // Recursively materialize (for transitive substitutions)
                    self.materialize(subst)
                } else {
                    ty.clone()
                }
            }

            InferType::Arrow(param, ret) => InferType::Arrow(
                Box::new(self.materialize(param)),
                Box::new(self.materialize(ret)),
            ),

            InferType::Generic { name, args } => InferType::Generic {
                name: name.clone(),
                args: args.iter().map(|a| self.materialize(a)).collect(),
            },

            InferType::Forall { vars, body } => InferType::Forall {
                vars: vars.clone(),
                body: Box::new(self.materialize(body)),
            },

            InferType::Concrete(_) => ty.clone(),
        }
    }

    /// Occurs check: does the type contain the variable?
    ///
    /// This prevents creating infinite types like α = α → β.
    fn contains(&self, ty: &InferType, var: &TypeVar) -> bool {
        let ty = self.materialize(ty);

        match ty {
            InferType::Var(v) => v == *var,
            InferType::Arrow(p, r) => self.contains(&p, var) || self.contains(&r, var),
            InferType::Generic { args, .. } => args.iter().any(|a| self.contains(a, var)),
            InferType::Forall { body, .. } => self.contains(&body, var),
            InferType::Concrete(_) => false,
        }
    }

    /// Get current substitutions
    pub fn substitutions(&self) -> &BTreeMap<TypeVar, InferType> {
        &self.substitutions
    }

    /// Apply current substitutions to a type
    pub fn apply(&self, ty: &InferType) -> InferType {
        self.materialize(ty)
    }
}

impl Default for Harmonizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::Type;

    #[test]
    fn test_harmonize_concrete() {
        let mut h = Harmonizer::new();
        let t1 = InferType::Concrete(Type::Number);
        let t2 = InferType::Concrete(Type::Number);

        assert!(h.harmonize(&t1, &t2).is_ok());
    }

    #[test]
    fn test_harmonize_var_concrete() {
        let mut h = Harmonizer::new();
        let alpha = TypeVar::fresh(0);
        let var_ty = InferType::Var(alpha.clone());
        let concrete_ty = InferType::Concrete(Type::Number);

        assert!(h.harmonize(&var_ty, &concrete_ty).is_ok());

        // Verify substitution
        let materialized = h.materialize(&var_ty);
        assert_eq!(materialized, InferType::Concrete(Type::Number));
    }

    #[test]
    fn test_harmonize_two_vars() {
        let mut h = Harmonizer::new();
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        assert!(h.harmonize(&InferType::Var(alpha.clone()), &InferType::Var(beta)).is_ok());

        // One should be substituted to the other
        assert_eq!(h.substitutions().len(), 1);
    }

    #[test]
    fn test_harmonize_arrow() {
        let mut h = Harmonizer::new();
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        let t1 = InferType::arrow(
            InferType::Var(alpha.clone()),
            InferType::Var(beta.clone()),
        );

        let t2 = InferType::arrow(
            InferType::Concrete(Type::Number),
            InferType::Concrete(Type::Text),
        );

        assert!(h.harmonize(&t1, &t2).is_ok());

        // Verify substitutions
        let alpha_materialized = h.materialize(&InferType::Var(alpha));
        let beta_materialized = h.materialize(&InferType::Var(beta));

        assert_eq!(alpha_materialized, InferType::Concrete(Type::Number));
        assert_eq!(beta_materialized, InferType::Concrete(Type::Text));
    }

    #[test]
    fn test_harmonize_infinite_type() {
        let mut h = Harmonizer::new();
        let alpha = TypeVar::fresh(0);

        // Try to unify α with α → β (infinite type)
        let t1 = InferType::Var(alpha.clone());
        let t2 = InferType::arrow(
            InferType::Var(alpha),
            InferType::Concrete(Type::Number),
        );

        let result = h.harmonize(&t1, &t2);
        assert!(result.is_err());

        match result {
            Err(TypeError::InfiniteType { .. }) => (),
            _ => panic!("Expected InfiniteType error"),
        }
    }

    #[test]
    fn test_harmonize_mismatch() {
        let mut h = Harmonizer::new();
        let t1 = InferType::Concrete(Type::Number);
        let t2 = InferType::Concrete(Type::Text);

        let result = h.harmonize(&t1, &t2);
        assert!(result.is_err());

        match result {
            Err(TypeError::Mismatch { .. }) => (),
            _ => panic!("Expected Mismatch error"),
        }
    }

    #[test]
    fn test_materialize_transitive() {
        let mut h = Harmonizer::new();
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        // α = β, β = Number
        h.harmonize(&InferType::Var(alpha.clone()), &InferType::Var(beta.clone()))
            .unwrap();
        h.harmonize(&InferType::Var(beta), &InferType::Concrete(Type::Number))
            .unwrap();

        // α should materialize to Number
        let result = h.materialize(&InferType::Var(alpha));
        assert_eq!(result, InferType::Concrete(Type::Number));
    }

    #[test]
    fn test_harmonize_list() {
        let mut h = Harmonizer::new();
        let alpha = TypeVar::fresh(0);

        let t1 = InferType::Generic {
            name: "List".to_string(),
            args: vec![InferType::Var(alpha.clone())],
        };

        let t2 = InferType::Generic {
            name: "List".to_string(),
            args: vec![InferType::Concrete(Type::Number)],
        };

        assert!(h.harmonize(&t1, &t2).is_ok());

        let materialized = h.materialize(&InferType::Var(alpha));
        assert_eq!(materialized, InferType::Concrete(Type::Number));
    }
}
