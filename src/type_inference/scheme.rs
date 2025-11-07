/// Type Schemes for Polymorphism
///
/// Type schemes represent polymorphic types using universal quantification (∀).
///
/// ## Examples
///
/// ```text
/// identity :: ∀α. α → α
/// map :: ∀α, β. List<α> → (α → β) → List<β>
/// ```
///
/// ## Let-Polymorphism
///
/// Let-bound values can be used polymorphically:
///
/// ```glimmer
/// chant identity(x) then yield x end
/// # Type: ∀T. T → T
///
/// bind a to identity(42)        # T = Number
/// bind b to identity("hello")   # T = Text (fresh instantiation)
/// ```
use crate::type_inference::{InferType, TypeVar};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

/// Type scheme with universal quantification
///
/// Represents polymorphic types of the form ∀α₁, α₂, ..., αₙ. τ
#[derive(Debug, Clone, PartialEq)]
pub struct TypeScheme {
    /// Universally quantified type variables (∀ these)
    pub quantified: Vec<TypeVar>,

    /// The type body
    pub body: InferType,
}

impl TypeScheme {
    /// Create a type scheme with quantified variables
    pub fn poly(quantified: Vec<TypeVar>, body: InferType) -> Self {
        TypeScheme { quantified, body }
    }

    /// Create a monomorphic type scheme (no quantification)
    pub fn mono(body: InferType) -> Self {
        TypeScheme {
            quantified: Vec::new(),
            body,
        }
    }

    /// Abstract over free type variables (generalization)
    ///
    /// Converts a monomorphic type to a polymorphic scheme by quantifying
    /// over all type variables that are free in the type but not in the
    /// environment.
    ///
    /// ## Example
    ///
    /// ```text
    /// Type: α → α
    /// Environment: {}
    /// Result: ∀α. α → α
    ///
    /// Type: α → β
    /// Environment: {x: β}
    /// Result: ∀α. α → β  (β not quantified - fixed in environment)
    /// ```
    pub fn abstract_type(ty: InferType, env_vars: &BTreeSet<TypeVar>) -> Self {
        let ty_vars = ty.free_vars();

        // Quantify over variables free in type but not in environment
        let mut quantified: Vec<_> = ty_vars.difference(env_vars).cloned().collect();
        quantified.sort(); // For deterministic output

        TypeScheme {
            quantified,
            body: ty,
        }
    }

    /// Specialize a polymorphic type (instantiation)
    ///
    /// Creates fresh type variables for all quantified variables, producing
    /// a monomorphic instance of the scheme.
    ///
    /// ## Example
    ///
    /// ```text
    /// Scheme: ∀α. α → α
    /// First call: β → β  (fresh β)
    /// Second call: γ → γ  (fresh γ)
    /// ```
    pub fn specialize(&self, next_var: &mut usize) -> InferType {
        if self.quantified.is_empty() {
            // Monomorphic - return as-is
            return self.body.clone();
        }

        let mut subst = BTreeMap::new();

        // Create fresh variables for each quantified variable
        for var in &self.quantified {
            let fresh = TypeVar::fresh(*next_var);
            *next_var += 1;
            subst.insert(var.clone(), InferType::Var(fresh));
        }

        // Apply substitution to body
        self.body.substitute(&subst)
    }

    /// Get all free type variables in this scheme
    ///
    /// Returns variables free in the body but not quantified.
    pub fn free_vars(&self) -> BTreeSet<TypeVar> {
        let mut free = self.body.free_vars();
        for var in &self.quantified {
            free.remove(var);
        }
        free
    }

    /// Check if this scheme is monomorphic (no quantification)
    pub fn is_mono(&self) -> bool {
        self.quantified.is_empty()
    }

    /// Check if this scheme is polymorphic (has quantification)
    pub fn is_poly(&self) -> bool {
        !self.quantified.is_empty()
    }
}

impl core::fmt::Display for TypeScheme {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.quantified.is_empty() {
            write!(f, "{}", self.body)
        } else {
            write!(f, "∀")?;
            for (i, var) in self.quantified.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", var)?;
            }
            write!(f, ". {}", self.body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::Type;

    #[test]
    fn test_mono_scheme() {
        let scheme = TypeScheme::mono(InferType::Concrete(Type::Number));
        assert!(scheme.is_mono());
        assert!(!scheme.is_poly());
        assert_eq!(scheme.quantified.len(), 0);
    }

    #[test]
    fn test_poly_scheme() {
        let alpha = TypeVar::fresh(0);
        let scheme = TypeScheme::poly(
            vec![alpha.clone()],
            InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(alpha)),
        );

        assert!(!scheme.is_mono());
        assert!(scheme.is_poly());
        assert_eq!(scheme.quantified.len(), 1);
    }

    #[test]
    fn test_abstract_type_empty_env() {
        let alpha = TypeVar::fresh(0);
        let ty = InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(alpha.clone()));

        let env_vars = BTreeSet::new();
        let scheme = TypeScheme::abstract_type(ty, &env_vars);

        // Should quantify over α
        assert_eq!(scheme.quantified.len(), 1);
        assert_eq!(scheme.quantified[0], alpha);
    }

    #[test]
    fn test_abstract_type_with_env() {
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        let ty = InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(beta.clone()));

        // β is in environment
        let mut env_vars = BTreeSet::new();
        env_vars.insert(beta.clone());

        let scheme = TypeScheme::abstract_type(ty, &env_vars);

        // Should only quantify over α (β is in environment)
        assert_eq!(scheme.quantified.len(), 1);
        assert_eq!(scheme.quantified[0], alpha);
    }

    #[test]
    fn test_specialize_mono() {
        let scheme = TypeScheme::mono(InferType::Concrete(Type::Number));
        let mut next_var = 0;

        let specialized = scheme.specialize(&mut next_var);
        assert_eq!(specialized, InferType::Concrete(Type::Number));
        assert_eq!(next_var, 0); // No fresh vars created
    }

    #[test]
    fn test_specialize_poly() {
        let alpha = TypeVar::fresh(0);
        let scheme = TypeScheme::poly(
            vec![alpha.clone()],
            InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(alpha)),
        );

        let mut next_var = 10;

        let specialized = scheme.specialize(&mut next_var);
        assert_eq!(next_var, 11); // One fresh var created

        // Should be β → β where β is fresh
        match specialized {
            InferType::Arrow(param, ret) => {
                assert_eq!(param, ret);
                match *param {
                    InferType::Var(var) => {
                        assert_eq!(var.id(), 10);
                    }
                    _ => panic!("Expected variable"),
                }
            }
            _ => panic!("Expected arrow type"),
        }
    }

    #[test]
    fn test_free_vars_scheme() {
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        // ∀α. α → β
        let scheme = TypeScheme::poly(
            vec![alpha.clone()],
            InferType::arrow(InferType::Var(alpha), InferType::Var(beta.clone())),
        );

        let free = scheme.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&beta));
    }

    #[test]
    fn test_scheme_display() {
        let alpha = TypeVar::fresh(0);
        let scheme = TypeScheme::poly(
            vec![alpha.clone()],
            InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(alpha)),
        );

        let display = format!("{}", scheme);
        assert!(display.contains("∀"));
        assert!(display.contains("α"));
    }
}
