/// Constraint Generation
///
/// Walks the AST and generates type requirements (constraints) that must
/// be satisfied for the program to be well-typed.
///
/// ## Process
///
/// 1. Assign a type variable to each expression
/// 2. Generate requirements based on usage:
///    - Function call: func_ty = arg_ty → return_ty
///    - Binary op: lhs_ty = rhs_ty = Number (for +, -, etc)
///    - If: condition_ty = Truth, then_ty = else_ty
/// 3. Collect all requirements into a RequirementSet
///
/// ## Example
///
/// ```glimmer
/// bind x to 42
/// bind y to x
/// ```
///
/// Generates:
/// - x :: α
/// - 42 :: Number
/// - Requirement: α = Number
/// - y :: β
/// - Requirement: β = α

use crate::ast::AstNode;
use crate::type_inference::{InferType, TypeVar, RequirementSet, TypeScheme};
use crate::type_inference::requirement::SourceLocation;
use crate::semantic::Type;
use std::collections::BTreeMap;

/// Constraint generator
pub struct ConstraintGenerator {
    /// Next available type variable ID
    next_var: usize,

    /// Accumulated requirements
    requirements: RequirementSet,

    /// Type environment (variable → type scheme)
    env: BTreeMap<String, TypeScheme>,
}

impl ConstraintGenerator {
    pub fn new() -> Self {
        ConstraintGenerator {
            next_var: 0,
            requirements: RequirementSet::new(),
            env: BTreeMap::new(),
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar::fresh(self.next_var);
        self.next_var += 1;
        var
    }

    /// Generate constraints from AST
    ///
    /// Returns the inferred type for the expression and accumulates
    /// requirements in self.requirements.
    pub fn infer_expr(&mut self, expr: &AstNode) -> InferType {
        match expr {
            // Literals have known types
            AstNode::Number(_) => InferType::Concrete(Type::Number),
            AstNode::Text(_) => InferType::Concrete(Type::Text),
            AstNode::Truth(_) => InferType::Concrete(Type::Truth),
            AstNode::Nothing => InferType::Concrete(Type::Nothing),

            // Variables: lookup in environment and instantiate
            AstNode::Ident(name) => {
                if let Some(scheme) = self.lookup(name).cloned() {
                    scheme.specialize(&mut self.next_var)
                } else {
                    // Undefined variable - generate fresh var
                    // Error will be caught later
                    InferType::Var(self.fresh_var())
                }
            }

            // Lists: infer element types and require homogeneity
            AstNode::List(elements) => {
                if elements.is_empty() {
                    // Empty list: List<α> for some fresh α
                    let elem_ty = InferType::Var(self.fresh_var());
                    InferType::Generic {
                        name: "List".to_string(),
                        args: vec![elem_ty],
                    }
                } else {
                    // Infer first element type
                    let first_ty = self.infer_expr(&elements[0]);

                    // Require all elements to match first
                    for elem in &elements[1..] {
                        let elem_ty = self.infer_expr(elem);
                        self.add_requirement(
                            elem_ty,
                            first_ty.clone(),
                            SourceLocation::unknown(),
                        );
                    }

                    InferType::Generic {
                        name: "List".to_string(),
                        args: vec![first_ty],
                    }
                }
            }

            // Binary operations
            AstNode::BinaryOp { op, left, right } => {
                use crate::ast::BinaryOperator;
                let left_ty = self.infer_expr(left);
                let right_ty = self.infer_expr(right);

                match op {
                    // Arithmetic: both must be Number, result is Number
                    BinaryOperator::Add
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Mod => {
                        self.add_requirement(
                            left_ty,
                            InferType::Concrete(Type::Number),
                            SourceLocation::unknown(),
                        );
                        self.add_requirement(
                            right_ty,
                            InferType::Concrete(Type::Number),
                            SourceLocation::unknown(),
                        );
                        InferType::Concrete(Type::Number)
                    }

                    // Comparison: both must match, result is Truth
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Less
                    | BinaryOperator::LessEq
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEq => {
                        self.add_requirement(left_ty, right_ty, SourceLocation::unknown());
                        InferType::Concrete(Type::Truth)
                    }

                    // Logical: both must be Truth, result is Truth
                    BinaryOperator::And | BinaryOperator::Or => {
                        self.add_requirement(
                            left_ty,
                            InferType::Concrete(Type::Truth),
                            SourceLocation::unknown(),
                        );
                        self.add_requirement(
                            right_ty,
                            InferType::Concrete(Type::Truth),
                            SourceLocation::unknown(),
                        );
                        InferType::Concrete(Type::Truth)
                    }
                }
            }

            // Unary operations
            AstNode::UnaryOp { op, operand } => {
                use crate::ast::UnaryOperator;
                let operand_ty = self.infer_expr(operand);

                match op {
                    UnaryOperator::Negate => {
                        self.add_requirement(
                            operand_ty,
                            InferType::Concrete(Type::Number),
                            SourceLocation::unknown(),
                        );
                        InferType::Concrete(Type::Number)
                    }
                    UnaryOperator::Not => {
                        self.add_requirement(
                            operand_ty,
                            InferType::Concrete(Type::Truth),
                            SourceLocation::unknown(),
                        );
                        InferType::Concrete(Type::Truth)
                    }
                }
            }

            // If statements
            AstNode::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_ty = self.infer_expr(condition);

                // Condition must be Truth
                self.add_requirement(
                    cond_ty,
                    InferType::Concrete(Type::Truth),
                    SourceLocation::unknown(),
                );

                // Infer then branch (sequence of statements)
                let then_ty = self.infer_block(then_branch);

                if let Some(else_br) = else_branch {
                    let else_ty = self.infer_block(else_br);
                    // Branches must match
                    self.add_requirement(then_ty.clone(), else_ty, SourceLocation::unknown());
                    then_ty
                } else {
                    // No else branch: result is Nothing if condition false
                    InferType::Concrete(Type::Nothing)
                }
            }

            // Function calls
            AstNode::Call { callee, args, type_args: _ } => {
                let func_ty = self.infer_expr(callee);

                // Infer argument types
                let arg_tys: Vec<_> = args.iter().map(|a| self.infer_expr(a)).collect();

                // Fresh variable for return type
                let return_ty = InferType::Var(self.fresh_var());

                // Require: func_ty = arg_tys → return_ty
                let expected_func_ty = self.arrow_from_args(arg_tys, return_ty.clone());
                self.add_requirement(func_ty, expected_func_ty, SourceLocation::unknown());

                return_ty
            }

            // Bind statements
            AstNode::BindStmt { name, value, typ: _ } => {
                let value_ty = self.infer_expr(value);

                // Add to environment as monomorphic (will generalize later)
                self.insert(name.clone(), TypeScheme::mono(value_ty.clone()));

                value_ty
            }

            // Block: infer each statement, return last
            AstNode::Block(statements) => {
                let mut last_ty = InferType::Concrete(Type::Nothing);

                for stmt in statements {
                    last_ty = self.infer_expr(stmt);
                }

                last_ty
            }

            // Default: return fresh variable
            _ => InferType::Var(self.fresh_var()),
        }
    }

    /// Infer type for a block of statements
    fn infer_block(&mut self, statements: &[AstNode]) -> InferType {
        let mut last_ty = InferType::Concrete(Type::Nothing);

        for stmt in statements {
            last_ty = self.infer_expr(stmt);
        }

        last_ty
    }

    /// Get accumulated requirements
    pub fn requirements(&self) -> &RequirementSet {
        &self.requirements
    }

    /// Take requirements, consuming the generator
    pub fn take_requirements(self) -> RequirementSet {
        self.requirements
    }

    /// Add a requirement
    pub fn add_requirement(&mut self, lhs: InferType, rhs: InferType, location: SourceLocation) {
        self.requirements.add(lhs, rhs, location);
    }

    /// Look up variable in environment
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.env.get(name)
    }

    /// Insert variable into environment
    pub fn insert(&mut self, name: String, scheme: TypeScheme) {
        self.env.insert(name, scheme);
    }

    /// Create arrow type from arguments
    fn arrow_from_args(&self, args: Vec<InferType>, return_ty: InferType) -> InferType {
        args.into_iter()
            .rev()
            .fold(return_ty, |acc, arg| InferType::arrow(arg, acc))
    }
}

impl Default for ConstraintGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_var_generation() {
        let mut gen = ConstraintGenerator::new();

        let v1 = gen.fresh_var();
        let v2 = gen.fresh_var();
        let v3 = gen.fresh_var();

        assert_eq!(v1.id(), 0);
        assert_eq!(v2.id(), 1);
        assert_eq!(v3.id(), 2);
    }

    #[test]
    fn test_arrow_from_args() {
        let gen = ConstraintGenerator::new();

        // Single arg: Number → Text
        let ty = gen.arrow_from_args(
            vec![InferType::Concrete(Type::Number)],
            InferType::Concrete(Type::Text),
        );

        match ty {
            InferType::Arrow(param, ret) => {
                assert_eq!(*param, InferType::Concrete(Type::Number));
                assert_eq!(*ret, InferType::Concrete(Type::Text));
            }
            _ => panic!("Expected arrow type"),
        }

        // Two args: Number → Text → Truth
        let ty = gen.arrow_from_args(
            vec![
                InferType::Concrete(Type::Number),
                InferType::Concrete(Type::Text),
            ],
            InferType::Concrete(Type::Truth),
        );

        // Should be curried: Number → (Text → Truth)
        match ty {
            InferType::Arrow(param1, rest) => {
                assert_eq!(*param1, InferType::Concrete(Type::Number));
                match *rest {
                    InferType::Arrow(param2, ret) => {
                        assert_eq!(*param2, InferType::Concrete(Type::Text));
                        assert_eq!(*ret, InferType::Concrete(Type::Truth));
                    }
                    _ => panic!("Expected nested arrow"),
                }
            }
            _ => panic!("Expected arrow type"),
        }
    }

    #[test]
    fn test_env_operations() {
        let mut gen = ConstraintGenerator::new();

        let scheme = TypeScheme::mono(InferType::Concrete(Type::Number));
        gen.insert("x".to_string(), scheme);

        assert!(gen.lookup("x").is_some());
        assert!(gen.lookup("y").is_none());
    }

    #[test]
    fn test_add_requirement() {
        let mut gen = ConstraintGenerator::new();

        let alpha = gen.fresh_var();
        gen.add_requirement(
            InferType::Var(alpha),
            InferType::Concrete(Type::Number),
            SourceLocation::unknown(),
        );

        assert_eq!(gen.requirements().len(), 1);
    }
}
