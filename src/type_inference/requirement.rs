/// Type Requirements (Constraints)
///
/// During type inference, we generate requirements that types must satisfy.
/// These are equations like "τ1 = τ2" that will be solved through harmonization.
///
/// ## Example
///
/// ```glimmer
/// bind x to 42
/// bind y to x
/// ```
///
/// Generates requirements:
/// - x :: α
/// - 42 :: Number
/// - α = Number (from binding)
/// - y :: β
/// - β = α (from binding)
///
/// Solution: α = β = Number

use crate::type_inference::InferType;

/// Location in source code for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        SourceLocation { line, column }
    }

    pub fn unknown() -> Self {
        SourceLocation { line: 0, column: 0 }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// A type requirement (constraint equation)
///
/// Represents the requirement that `lhs` must equal `rhs`.
#[derive(Debug, Clone, PartialEq)]
pub struct Requirement {
    pub lhs: InferType,
    pub rhs: InferType,
    pub location: SourceLocation,
}

impl Requirement {
    pub fn new(lhs: InferType, rhs: InferType, location: SourceLocation) -> Self {
        Requirement { lhs, rhs, location }
    }

    pub fn at(lhs: InferType, rhs: InferType, line: usize, column: usize) -> Self {
        Requirement {
            lhs,
            rhs,
            location: SourceLocation::new(line, column),
        }
    }
}

/// Collection of type requirements
#[derive(Debug, Clone)]
pub struct RequirementSet {
    requirements: Vec<Requirement>,
}

impl RequirementSet {
    pub fn new() -> Self {
        RequirementSet {
            requirements: Vec::new(),
        }
    }

    pub fn add(&mut self, lhs: InferType, rhs: InferType, location: SourceLocation) {
        self.requirements.push(Requirement::new(lhs, rhs, location));
    }

    pub fn add_requirement(&mut self, req: Requirement) {
        self.requirements.push(req);
    }

    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }

    pub fn len(&self) -> usize {
        self.requirements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }

    /// Merge another requirement set into this one
    pub fn extend(&mut self, other: RequirementSet) {
        self.requirements.extend(other.requirements);
    }
}

impl Default for RequirementSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::Type;
    use crate::type_inference::TypeVar;

    #[test]
    fn test_requirement_creation() {
        let alpha = TypeVar::fresh(0);
        let req = Requirement::new(
            InferType::Var(alpha),
            InferType::Concrete(Type::Number),
            SourceLocation::new(1, 5),
        );

        assert_eq!(req.location.line, 1);
        assert_eq!(req.location.column, 5);
    }

    #[test]
    fn test_requirement_set() {
        let mut set = RequirementSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);

        let alpha = TypeVar::fresh(0);
        set.add(
            InferType::Var(alpha),
            InferType::Concrete(Type::Number),
            SourceLocation::new(1, 5),
        );

        assert!(!set.is_empty());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_requirement_set_extend() {
        let mut set1 = RequirementSet::new();
        let mut set2 = RequirementSet::new();

        let alpha = TypeVar::fresh(0);
        let beta = TypeVar::fresh(1);

        set1.add(
            InferType::Var(alpha),
            InferType::Concrete(Type::Number),
            SourceLocation::unknown(),
        );

        set2.add(
            InferType::Var(beta),
            InferType::Concrete(Type::Text),
            SourceLocation::unknown(),
        );

        set1.extend(set2);
        assert_eq!(set1.len(), 2);
    }

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(10, 5);
        assert_eq!(format!("{}", loc), "line 10, column 5");
    }
}
