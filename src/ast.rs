//! # Abstract Syntax Tree (AST)
//!
//! Defines the structure of parsed Glimmer-Weave programs.
//!
//! The AST represents the syntactic structure of Glimmer-Weave code,
//! capturing statements, expressions, and their relationships.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::source_location::SourceSpan;

/// Borrow mode for parameters and types
///
/// Specifies how ownership is handled when passing values or accessing data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum BorrowMode {
    /// Owned: Takes ownership (move semantics)
    /// Example: `chant consume(data as List<Number>)`
    #[default]
    Owned,

    /// Borrowed: Shared/immutable borrow (read-only access)
    /// Example: `chant read(borrow data as List<Number>)`
    Borrowed,

    /// BorrowedMut: Mutable borrow (exclusive write access)
    /// Example: `chant modify(borrow mut data as List<Number>)`
    BorrowedMut,
}


/// Lifetime annotation
///
/// Tracks how long a reference remains valid.
/// Examples: 'span, 'a, 'static
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lifetime {
    pub name: String,
}

impl Lifetime {
    /// Create a new lifetime annotation
    pub fn new(name: impl Into<String>) -> Self {
        Lifetime { name: name.into() }
    }

    /// The 'static lifetime (valid for entire program duration)
    pub fn static_lifetime() -> Self {
        Lifetime { name: "static".to_string() }
    }
}

/// Type annotation in the AST (syntactic representation)
///
/// This is the syntactic form of types as they appear in source code.
/// The semantic analyzer converts these to semantic::Type for type checking.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    /// Simple type name: `Number`, `Text`, `Truth`
    Named(String),
    /// Generic type parameter: `T`, `U`, `Key`, `Value`
    /// Used in function/struct definitions to represent type variables
    Generic(String),
    /// Parametrized type: `Box<Number>`, `Pair<T, U>`
    /// The first String is the type constructor name, Vec contains type arguments
    Parametrized {
        name: String,
        type_args: Vec<TypeAnnotation>,
    },
    /// List type: `List<Number>` (legacy support, equivalent to Parametrized)
    List(Box<TypeAnnotation>),
    /// Map type: `Map`
    Map,
    /// Function type: `Function<(Number, Text) -> Truth>`
    Function {
        param_types: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },
    /// Optional/nullable type: `Number?` (future feature)
    Optional(Box<TypeAnnotation>),
    /// Borrowed type: `borrow T`, `borrow mut T`, `borrow 'a T`
    /// Represents a reference to a value (immutable or mutable)
    Borrowed {
        lifetime: Option<Lifetime>,
        inner: Box<TypeAnnotation>,
        mutable: bool,
    },
}

/// Function parameter with optional type annotation
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub typ: Option<TypeAnnotation>,
    /// If true, this parameter collects remaining arguments into a list
    /// Syntax: `...rest` or `...args`
    /// Must be the last parameter if present
    pub is_variadic: bool,
    /// Ownership mode: Owned, Borrowed, or BorrowedMut
    pub borrow_mode: BorrowMode,
    /// Optional lifetime annotation for borrowed parameters
    pub lifetime: Option<Lifetime>,
}

/// Struct field definition
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub typ: TypeAnnotation,
}

/// Enum variant case definition
///
/// Represents a single case in an enum variant definition.
/// Can be a simple unit variant (no fields) or carry data (fields).
///
/// Examples:
/// - `Red` - unit variant (fields is empty)
/// - `Move(x: Number, y: Number)` - variant with data
#[derive(Debug, Clone, PartialEq)]
pub struct VariantCase {
    pub name: String,
    pub fields: Vec<Parameter>,  // Fields if this variant carries data
}

/// Trait method signature
///
/// Represents a method signature in a trait definition.
/// Unlike regular functions, trait methods are declarations without implementations.
///
/// Examples:
/// - `chant show(self) -> Text` - method returning Text
/// - `chant add(self, item: T)` - method with parameter
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Parameter>,  // First parameter must be 'self'
    pub return_type: Option<TypeAnnotation>,
}

impl Parameter {
    /// Create a parameter without type annotation (for backward compatibility)
    pub fn untyped(name: String) -> Self {
        Parameter {
            name,
            typ: None,
            is_variadic: false,
            borrow_mode: BorrowMode::Owned,
            lifetime: None,
        }
    }

    /// Create a parameter with type annotation
    pub fn typed(name: String, typ: TypeAnnotation) -> Self {
        Parameter {
            name,
            typ: Some(typ),
            is_variadic: false,
            borrow_mode: BorrowMode::Owned,
            lifetime: None,
        }
    }

    /// Create a variadic parameter
    pub fn variadic(name: String, typ: Option<TypeAnnotation>) -> Self {
        Parameter {
            name,
            typ,
            is_variadic: true,
            borrow_mode: BorrowMode::Owned,
            lifetime: None,
        }
    }

    /// Create an immutably borrowed parameter: `borrow x as T` or `borrow 'a x as T`
    pub fn borrowed(name: String, typ: TypeAnnotation, lifetime: Option<Lifetime>) -> Self {
        Parameter {
            name,
            typ: Some(typ),
            is_variadic: false,
            borrow_mode: BorrowMode::Borrowed,
            lifetime,
        }
    }

    /// Create a mutably borrowed parameter: `borrow mut x as T` or `borrow 'a mut x as T`
    pub fn borrowed_mut(name: String, typ: TypeAnnotation, lifetime: Option<Lifetime>) -> Self {
        Parameter {
            name,
            typ: Some(typ),
            is_variadic: false,
            borrow_mode: BorrowMode::BorrowedMut,
            lifetime,
        }
    }
}

/// A node in the Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    // === Statements ===

    /// Immutable binding: `bind x to 42` or `bind x: Number to 42`
    BindStmt {
        name: String,
        typ: Option<TypeAnnotation>,
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Mutable variable: `weave counter as 0` or `weave counter: Number as 0`
    WeaveStmt {
        name: String,
        typ: Option<TypeAnnotation>,
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Mutation: `set counter to 10`, `set list[i] to 5`, `set obj.field to "value"`
    SetStmt {
        target: Box<AstNode>,  // Can be Ident, Index, or FieldAccess
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Conditional: `should x > 5 then ... otherwise ... end`
    IfStmt {
        condition: Box<AstNode>,
        then_branch: Vec<AstNode>,
        else_branch: Option<Vec<AstNode>>,
        span: SourceSpan,
    },

    /// Bounded loop: `for each x in list then ... end`
    ForStmt {
        variable: String,
        iterable: Box<AstNode>,
        body: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Unbounded loop: `whilst condition then ... end`
    WhileStmt {
        condition: Box<AstNode>,
        body: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Function definition: `chant greet(name) then ... end`
    /// or with types: `chant factorial(n: Number) -> Number then ... end`
    /// or with generics: `chant identity<T>(x: T) -> T then ... end`
    /// or with lifetimes: `chant process<'a>(borrow 'a data as List<T>) -> borrow 'a T then ... end`
    ChantDef {
        name: String,
        type_params: Vec<String>,  // Generic type parameters like ["T", "U"]
        lifetime_params: Vec<Lifetime>,  // Lifetime parameters like ['a, 'b]
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Struct definition: `form Person with name as Text age as Number end`
    /// or with generics: `form Box<T> with value as T end`
    FormDef {
        name: String,
        type_params: Vec<String>,  // Generic type parameters like ["T", "U"]
        fields: Vec<StructField>,
        span: SourceSpan,
    },

    /// Enum definition: `variant Color then Red, Green, Blue end`
    /// or with data: `variant Message then Quit, Move(x: Number, y: Number) end`
    /// or with generics: `variant Option<T> then Some(value: T), None end`
    VariantDef {
        name: String,
        type_params: Vec<String>,  // Generic type parameters like ["T"]
        variants: Vec<VariantCase>,
        span: SourceSpan,
    },

    /// Trait definition: `aspect Display then chant show(self) -> Text end`
    /// or with generics: `aspect Container<T> then chant add(self, item: T) end`
    AspectDef {
        name: String,
        type_params: Vec<String>,  // Generic type parameters like ["T"]
        methods: Vec<TraitMethod>,
        span: SourceSpan,
    },

    /// Trait implementation: `embody Display for Number then chant show(self) -> Text then ... end end`
    /// or with generic trait: `embody Container<Number> for NumberList then ... end`
    EmbodyStmt {
        aspect_name: String,
        type_args: Vec<TypeAnnotation>,  // Type arguments for generic traits
        target_type: TypeAnnotation,
        methods: Vec<AstNode>,  // ChantDef nodes
        span: SourceSpan,
    },

    /// Return statement: `yield result`
    YieldStmt {
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Pattern matching: `match x with when 1 then ... end`
    MatchStmt {
        value: Box<AstNode>,
        arms: Vec<MatchArm>,
        span: SourceSpan,
    },

    /// Error handling: `attempt ... harmonize on Error then ... end`
    AttemptStmt {
        body: Vec<AstNode>,
        handlers: Vec<ErrorHandler>,
        span: SourceSpan,
    },

    /// Capability request: `request VGA.write with justification "message"`
    RequestStmt {
        capability: Box<AstNode>,
        justification: String,
        span: SourceSpan,
    },

    // === Module System ===

    /// Module declaration: `grove Math with body end`
    ModuleDecl {
        name: String,
        body: Vec<AstNode>,
        exports: Vec<String>,  // Items listed in 'offer'
        span: SourceSpan,
    },

    /// Import statement: `summon Math from "std/math.gw"`
    /// or selective: `gather sqrt, pow from Math`
    Import {
        module_name: String,
        path: String,
        items: Option<Vec<String>>,  // None = import all (summon), Some = specific items (gather)
        alias: Option<String>,        // Optional 'as' alias
        span: SourceSpan,
    },

    /// Export statement: `offer sqrt, pow`
    Export {
        items: Vec<String>,
        span: SourceSpan,
    },

    // === Expressions ===

    /// Numeric literal: `42`, `3.14`
    Number {
        value: f64,
        span: SourceSpan,
    },

    /// String literal: `"hello"`
    Text {
        value: String,
        span: SourceSpan,
    },

    /// Boolean literal: `true`, `false`
    Truth {
        value: bool,
        span: SourceSpan,
    },

    /// Null/void value: `nothing`
    Nothing {
        span: SourceSpan,
    },

    /// Variable reference: `x`, `counter`
    Ident {
        name: String,
        span: SourceSpan,
    },

    /// Triumph value: `Triumph(42)` (successful Outcome)
    Triumph {
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Mishap value: `Mishap("error")` (failed Outcome)
    Mishap {
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Present value: `Present(42)` (Maybe with value)
    Present {
        value: Box<AstNode>,
        span: SourceSpan,
    },

    /// Absent value: `Absent` (Maybe without value)
    Absent {
        span: SourceSpan,
    },

    /// List literal: `[1, 2, 3]`
    List {
        elements: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Map literal: `{name: "Elara", age: 42}`
    Map {
        entries: Vec<(String, AstNode)>,
        span: SourceSpan,
    },

    /// Struct literal: `Person { name: "Alice", age: 30 }`
    /// or with type args: `Box<Number> { value: 42 }`
    StructLiteral {
        struct_name: String,
        type_args: Vec<TypeAnnotation>,  // Type arguments for generic instantiation
        fields: Vec<(String, AstNode)>,
        span: SourceSpan,
    },

    /// Binary operation: `x + y`, `a > b`
    BinaryOp {
        left: Box<AstNode>,
        op: BinaryOperator,
        right: Box<AstNode>,
        span: SourceSpan,
    },

    /// Unary operation: `not x`, `-y`
    UnaryOp {
        op: UnaryOperator,
        operand: Box<AstNode>,
        span: SourceSpan,
    },

    /// Borrow expression: `borrow x`, `borrow mut y`
    /// Creates a reference to a value
    BorrowExpr {
        value: Box<AstNode>,
        mutable: bool,
        span: SourceSpan,
    },

    /// Function call: `greet("Elara")`, `VGA.write("Hello")`
    /// or with type args: `identity<Number>(42)`
    Call {
        callee: Box<AstNode>,
        type_args: Vec<TypeAnnotation>,  // Type arguments for generic function calls
        args: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Field access: `person.name`, `VGA.write`
    FieldAccess {
        object: Box<AstNode>,
        field: String,
        span: SourceSpan,
    },

    /// Module-qualified access: `Math.sqrt`, `Collections.List`
    ModuleAccess {
        module: String,
        member: String,
        span: SourceSpan,
    },

    /// Index access: `list[0]`
    IndexAccess {
        object: Box<AstNode>,
        index: Box<AstNode>,
        span: SourceSpan,
    },

    /// Range: `range(1, 10)`
    Range {
        start: Box<AstNode>,
        end: Box<AstNode>,
        span: SourceSpan,
    },

    /// Pipeline: `x | filter by y > 5 | take 10`
    Pipeline {
        stages: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Query expression: `seek where essence is "Scroll"`
    SeekExpr {
        conditions: Vec<QueryCondition>,
        span: SourceSpan,
    },

    /// Expression statement (for side effects)
    ExprStmt {
        expr: Box<AstNode>,
        span: SourceSpan,
    },

    /// Block of statements
    Block {
        statements: Vec<AstNode>,
        span: SourceSpan,
    },

    /// Break statement: exits innermost loop
    Break {
        span: SourceSpan,
    },

    /// Continue statement: skip to next iteration of innermost loop
    Continue {
        span: SourceSpan,
    },

    /// Try operator: `expr?` - propagates Mishap errors, unwraps Triumph
    Try {
        expr: Box<AstNode>,
        span: SourceSpan,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Mod,      // %

    // Comparison
    Equal,    // is
    NotEqual, // is not
    Greater,  // >
    Less,     // <
    GreaterEq, // >=
    LessEq,   // <=

    // Logical
    And,      // and
    Or,       // or
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,     // not
    Negate,  // -
}

/// Match arm: `when pattern then body`
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<AstNode>,
}

/// Pattern for pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal pattern: `when 42 then ...`
    Literal(AstNode),
    /// Variable binding pattern: `when x then ...`
    Ident(String),
    /// Wildcard pattern: `otherwise`
    Wildcard,
    /// Enum pattern: `when Triumph(x) then ...` or `when Absent then ...`
    Enum {
        variant: String,  // "Triumph", "Mishap", "Present", "Absent"
        inner: Option<Box<Pattern>>,  // The inner pattern (if any)
    },
}

/// Error handler: `harmonize on ErrorType then ...`
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorHandler {
    pub error_type: String,
    pub body: Vec<AstNode>,
}

/// Query condition for seek expressions
#[derive(Debug, Clone, PartialEq)]
pub struct QueryCondition {
    pub field: String,
    pub operator: QueryOperator,
    pub value: Box<AstNode>,
}

/// Query operators for World-Tree queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryOperator {
    Is,           // is
    IsNot,        // is not
    Greater,      // >
    Less,         // <
    GreaterEq,    // >=
    LessEq,       // <=
    After,        // after (temporal)
    Before,       // before (temporal)
}

impl AstNode {
    /// Check if this node is a statement
    pub fn is_statement(&self) -> bool {
        matches!(
            self,
            AstNode::BindStmt { .. }
                | AstNode::WeaveStmt { .. }
                | AstNode::SetStmt { .. }
                | AstNode::IfStmt { .. }
                | AstNode::ForStmt { .. }
                | AstNode::WhileStmt { .. }
                | AstNode::ChantDef { .. }
                | AstNode::FormDef { .. }
                | AstNode::VariantDef { .. }
                | AstNode::AspectDef { .. }
                | AstNode::EmbodyStmt { .. }
                | AstNode::YieldStmt { .. }
                | AstNode::MatchStmt { .. }
                | AstNode::AttemptStmt { .. }
                | AstNode::RequestStmt { .. }
                | AstNode::ExprStmt { .. }
        )
    }

    /// Check if this node is an expression
    pub fn is_expression(&self) -> bool {
        !self.is_statement()
    }
}

impl BinaryOperator {
    /// Get the precedence of this operator (higher = tighter binding)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Greater
            | BinaryOperator::Less
            | BinaryOperator::GreaterEq
            | BinaryOperator::LessEq => 3,
            BinaryOperator::Add | BinaryOperator::Sub => 4,
            BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Mod => 5,
        }
    }
}
