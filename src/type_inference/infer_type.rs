/// Inference Types
///
/// Extended type representation used during type inference. This builds on top
/// of the existing `Type` enum from semantic analysis.
///
/// ## Differences from `Type`
///
/// - **Type** - Concrete types known at analysis time
/// - **InferType** - Types during inference (may contain variables)
///
/// ## Key Features
///
/// - **Type Variables** - `Var(α)` represents unknowns
/// - **Quantification** - `Forall` for polymorphic types (∀T. T → T)
/// - **Convertible** - Can convert between `Type` and `InferType`
use crate::semantic::Type;
use crate::type_inference::TypeVar;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::fmt;

/// Type representation during inference
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferType {
    /// Concrete type (Number, Text, etc.)
    Concrete(Type),

    /// Type variable (unknown type)
    Var(TypeVar),

    /// Function type (param → return)
    Arrow(Box<InferType>, Box<InferType>),

    /// Generic type constructor (List<T>, Map<K, V>)
    Generic {
        name: String,
        args: Vec<InferType>,
    },

    /// Polymorphic type (∀α. α → α)
    ///
    /// Used for let-polymorphism:
    /// ```glimmer
    /// chant identity<T>(x: T) -> T then yield x end
    /// # Type: ∀T. T → T
    /// ```
    Forall {
        vars: Vec<TypeVar>,
        body: Box<InferType>,
    },
}

impl InferType {
    /// Create a concrete type
    pub fn concrete(ty: Type) -> Self {
        InferType::Concrete(ty)
    }

    /// Create a type variable
    pub fn var(var: TypeVar) -> Self {
        InferType::Var(var)
    }

    /// Create a function type
    pub fn arrow(param: InferType, ret: InferType) -> Self {
        InferType::Arrow(Box::new(param), Box::new(ret))
    }

    /// Create a generic type
    pub fn generic(name: String, args: Vec<InferType>) -> Self {
        InferType::Generic { name, args }
    }

    /// Create a polymorphic type
    pub fn forall(vars: Vec<TypeVar>, body: InferType) -> Self {
        InferType::Forall {
            vars,
            body: Box::new(body),
        }
    }

    /// Get all free type variables (not bound by ∀)
    pub fn free_vars(&self) -> BTreeSet<TypeVar> {
        match self {
            InferType::Concrete(_) => BTreeSet::new(),
            InferType::Var(v) => {
                let mut set = BTreeSet::new();
                set.insert(v.clone());
                set
            }
            InferType::Arrow(param, ret) => {
                let mut vars = param.free_vars();
                vars.extend(ret.free_vars());
                vars
            }
            InferType::Generic { args, .. } => {
                let mut vars = BTreeSet::new();
                for arg in args {
                    vars.extend(arg.free_vars());
                }
                vars
            }
            InferType::Forall { vars: bound, body } => {
                let mut free = body.free_vars();
                for var in bound {
                    free.remove(var);
                }
                free
            }
        }
    }

    /// Apply substitutions (materialize concrete types)
    pub fn substitute(&self, subst: &BTreeMap<TypeVar, InferType>) -> InferType {
        match self {
            InferType::Concrete(_) => self.clone(),
            InferType::Var(var) => {
                if let Some(ty) = subst.get(var) {
                    ty.substitute(subst)
                } else {
                    self.clone()
                }
            }
            InferType::Arrow(param, ret) => {
                InferType::Arrow(
                    Box::new(param.substitute(subst)),
                    Box::new(ret.substitute(subst)),
                )
            }
            InferType::Generic { name, args } => InferType::Generic {
                name: name.clone(),
                args: args.iter().map(|a| a.substitute(subst)).collect(),
            },
            InferType::Forall { vars, body } => {
                // Don't substitute bound variables
                let mut filtered_subst = subst.clone();
                for var in vars {
                    filtered_subst.remove(var);
                }
                InferType::Forall {
                    vars: vars.clone(),
                    body: Box::new(body.substitute(&filtered_subst)),
                }
            }
        }
    }

    /// Convert to concrete Type (fails if type variables remain)
    pub fn to_concrete(&self) -> Option<Type> {
        match self {
            InferType::Concrete(ty) => Some(ty.clone()),
            InferType::Var(_) => None, // Can't convert unsolved variable
            InferType::Arrow(param, ret) => {
                let param_ty = param.to_concrete()?;
                let ret_ty = ret.to_concrete()?;
                Some(Type::Function {
                    params: vec![param_ty],
                    return_type: Box::new(ret_ty),
                })
            }
            InferType::Generic { name, args } => {
                let concrete_args: Option<Vec<_>> =
                    args.iter().map(|a| a.to_concrete()).collect();
                let concrete_args = concrete_args?;

                match name.as_str() {
                    "List" if args.len() == 1 => {
                        Some(Type::List(Box::new(concrete_args[0].clone())))
                    }
                    _ => Some(Type::Generic {
                        name: name.clone(),
                        type_args: concrete_args,
                    }),
                }
            }
            InferType::Forall { body, .. } => {
                // For now, just convert body (should be instantiated first)
                body.to_concrete()
            }
        }
    }

    /// Display in natural language for error messages
    pub fn display_natural(&self) -> String {
        match self {
            InferType::Concrete(Type::Number) => "a number".to_string(),
            InferType::Concrete(Type::Text) => "text".to_string(),
            InferType::Concrete(Type::Truth) => "a truth value".to_string(),
            InferType::Concrete(Type::Nothing) => "nothing".to_string(),
            InferType::Concrete(Type::List(inner)) => {
                format!("a list of {}", InferType::Concrete(*inner.clone()).display_natural())
            }
            InferType::Var(var) => {
                format!("an unknown type ({})", var.name())
            }
            InferType::Arrow(param, ret) => {
                format!(
                    "a function taking {} and yielding {}",
                    param.display_natural(),
                    ret.display_natural()
                )
            }
            InferType::Generic { name, args } if name == "List" && args.len() == 1 => {
                format!("a list of {}", args[0].display_natural())
            }
            InferType::Generic { name, args } => {
                if args.is_empty() {
                    format!("a {}", name)
                } else {
                    let arg_strs: Vec<_> = args.iter()
                        .map(|a| a.display_natural())
                        .collect();
                    format!("a {}<{}>", name, arg_strs.join(", "))
                }
            }
            InferType::Forall { vars, body } => {
                let var_names: Vec<_> = vars.iter().map(|v| v.name()).collect();
                format!("∀{}. {}", var_names.join(", "), body.display_natural())
            }
            InferType::Concrete(ty) => format!("{:?}", ty),
        }
    }
}

impl From<Type> for InferType {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Function { params, return_type } => {
                // Convert multi-param function to curried arrows
                let mut result = InferType::Concrete(*return_type);
                for param in params.into_iter().rev() {
                    result = InferType::arrow(InferType::Concrete(param), result);
                }
                result
            }
            Type::List(inner) => InferType::Generic {
                name: "List".to_string(),
                args: vec![InferType::Concrete(*inner)],
            },
            Type::Generic { name, type_args } => InferType::Generic {
                name,
                args: type_args.into_iter().map(InferType::Concrete).collect(),
            },
            other => InferType::Concrete(other),
        }
    }
}

impl fmt::Display for InferType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferType::Concrete(ty) => write!(f, "{:?}", ty),
            InferType::Var(var) => write!(f, "{}", var),
            InferType::Arrow(param, ret) => write!(f, "({} → {})", param, ret),
            InferType::Generic { name, args } => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}<", name)?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")
                }
            }
            InferType::Forall { vars, body } => {
                write!(f, "∀")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, ". {}", body)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_vars_simple() {
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        // α has free var α
        let ty = InferType::Var(alpha.clone());
        let free = ty.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&alpha));

        // α → β has free vars α, β
        let ty = InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(beta.clone()));
        let free = ty.free_vars();
        assert_eq!(free.len(), 2);
        assert!(free.contains(&alpha));
        assert!(free.contains(&beta));
    }

    #[test]
    fn test_free_vars_forall() {
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        // ∀α. α → β has free var β (α is bound)
        let ty = InferType::forall(
            vec![alpha.clone()],
            InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(beta.clone())),
        );
        let free = ty.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&beta));
        assert!(!free.contains(&alpha));
    }

    #[test]
    fn test_substitute_var() {
        let alpha = TypeVar::fresh(0);
        let ty = InferType::Var(alpha.clone());

        let mut subst = BTreeMap::new();
        subst.insert(alpha, InferType::Concrete(Type::Number));

        let result = ty.substitute(&subst);
        assert_eq!(result, InferType::Concrete(Type::Number));
    }

    #[test]
    fn test_substitute_arrow() {
        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        // α → β
        let ty = InferType::arrow(InferType::Var(alpha.clone()), InferType::Var(beta.clone()));

        let mut subst = BTreeMap::new();
        subst.insert(alpha, InferType::Concrete(Type::Number));
        subst.insert(beta, InferType::Concrete(Type::Text));

        let result = ty.substitute(&subst);
        let expected = InferType::arrow(
            InferType::Concrete(Type::Number),
            InferType::Concrete(Type::Text),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_concrete_success() {
        let ty = InferType::Concrete(Type::Number);
        assert_eq!(ty.to_concrete(), Some(Type::Number));

        let ty = InferType::arrow(
            InferType::Concrete(Type::Number),
            InferType::Concrete(Type::Text),
        );
        if let Some(Type::Function { params, return_type }) = ty.to_concrete() {
            assert_eq!(params, vec![Type::Number]);
            assert_eq!(*return_type, Type::Text);
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_to_concrete_failure() {
        let alpha = TypeVar::fresh(0);
        let ty = InferType::Var(alpha);
        assert_eq!(ty.to_concrete(), None);
    }

    #[test]
    fn test_display_natural() {
        let ty = InferType::Concrete(Type::Number);
        assert_eq!(ty.display_natural(), "a number");

        let alpha = TypeVar::fresh(0);
        let ty = InferType::Var(alpha);
        assert_eq!(ty.display_natural(), "an unknown type (α)");

        let ty = InferType::arrow(
            InferType::Concrete(Type::Number),
            InferType::Concrete(Type::Text),
        );
        assert_eq!(
            ty.display_natural(),
            "a function taking a number and yielding text"
        );
    }

    #[test]
    fn test_from_type() {
        let ty = Type::Number;
        let infer_ty = InferType::from(ty);
        assert_eq!(infer_ty, InferType::Concrete(Type::Number));

        let ty = Type::List(Box::new(Type::Number));
        let infer_ty = InferType::from(ty);
        assert_eq!(
            infer_ty,
            InferType::Generic {
                name: "List".to_string(),
                args: vec![InferType::Concrete(Type::Number)]
            }
        );
    }
}
