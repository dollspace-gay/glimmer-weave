//! # Glimmer-Weave
//!
//! A scripting language for AethelOS that emphasizes natural expression,
//! query-first design, and capability-based security.
//!
//! ## Philosophy
//!
//! Glimmer-Weave is the living voice of AethelOS. Like water flowing through stone,
//! scripts are ephemeral but powerful. They exist to serve the moment, not eternity.
//!
//! ## Features
//!
//! - **Natural Expression**: Syntax that reads like intention (`bind`, `should`, `chant`)
//! - **Query-First**: Native support for World-Tree queries (`seek where...`)
//! - **Capability-Aware**: Security built into the language (`request VGA.write`)
//! - **Harmonic Failure**: Errors are suggestions, not crashes (`attempt`/`harmonize`)
//! - **Contextual Flow**: State flows naturally through pipelines (`|`)
//!
//! ## Example
//!
//! ```glimmer-weave
//! bind name to "Elara"
//! bind age to 42
//!
//! should age >= 18 then
//!     VGA.write("Welcome, " + name)
//! otherwise
//!     VGA.write("Access denied")
//! end
//! ```
//!
//! ## Modules
//!
//! - [`token`]: Token definitions for the lexer
//! - [`lexer`]: Tokenizer for Glimmer-Weave source code
//! - [`ast`]: Abstract Syntax Tree node types
//! - [`parser`]: Parser for building AST from tokens
//! - [`eval`]: Evaluator/interpreter for executing AST
//! - [`codegen`]: Code generator for compiling to x86-64 assembly

// Declare as no_std by default, but allow std feature to enable standard library
#![cfg_attr(not(feature = "std"), no_std)]

// When std feature is enabled, provide alloc via std
// Import macros (format!, vec!) from alloc
#[cfg(feature = "std")]
#[macro_use]
extern crate std as alloc;

// When std feature is disabled, use the real alloc crate for heap allocations
// Import macros (format!, vec!) from alloc
#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

// Prelude module with common alloc types for no_std compatibility
mod prelude;

pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod eval;
pub mod codegen;
pub mod elf;
pub mod runtime;
pub mod semantic;
pub mod bytecode;
pub mod bytecode_compiler;
pub mod vm;
pub mod monomorphize;
pub mod type_inference;

// Re-export commonly used types
pub use token::{Token, Span};
pub use lexer::Lexer;
pub use ast::{AstNode, BinaryOperator, UnaryOperator, TypeAnnotation, Parameter, VariantCase};
pub use parser::{Parser, ParseError, ParseResult};
pub use eval::{Value, RuntimeError, Environment, Evaluator};
pub use codegen::{CodeGen, Instruction, Register, compile_to_asm};
pub use elf::{ElfBuilder, create_elf_object};
pub use semantic::{SemanticAnalyzer, SemanticError, Type, analyze};
