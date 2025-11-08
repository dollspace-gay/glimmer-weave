//! # Native Runtime Support for x86-64 Code Generation
//!
//! Provides heap allocation and memory management primitives for generated
//! x86-64 assembly code.
//!
//! ## Design Philosophy
//!
//! The native runtime uses a custom free-list allocator (`gl_malloc`/`gl_free`)
//! implemented in x86-64 assembly. This allows:
//! - Struct instantiation on the heap
//! - Dynamic string allocation
//! - Closure environment capture
//! - List and map allocation
//!
//! The allocator is completely self-contained and does not depend on libc,
//! making it suitable for AethelOS deployment and other no_std environments.
//!
//! ## Memory Layout
//!
//! ### Structs
//! Structs are allocated on the heap with fields laid out sequentially:
//! ```text
//! +------------------+
//! | field_0 (8 bytes)|
//! +------------------+
//! | field_1 (8 bytes)|
//! +------------------+
//! | ...              |
//! +------------------+
//! ```
//!
//! All fields are 8-byte aligned (f64 or pointer-sized).
//!
//! ### Strings
//! Strings are allocated with a length prefix:
//! ```text
//! +------------------+
//! | length (8 bytes) |
//! +------------------+
//! | char data ...    |
//! +------------------+
//! ```
//!
//! ### Lists
//! Lists use a capacity + length + data layout:
//! ```text
//! +------------------+
//! | capacity (8 bytes)|
//! +------------------+
//! | length (8 bytes) |
//! +------------------+
//! | elements ...     |
//! +------------------+
//! ```

use crate::codegen::Instruction;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

/// Native runtime functions available to generated code
pub struct NativeRuntime {
    /// Generated initialization code (data section, etc)
    pub init_code: Vec<Instruction>,
}

impl NativeRuntime {
    /// Create a new native runtime
    pub fn new() -> Self {
        NativeRuntime {
            init_code: Vec::new(),
        }
    }

    /// Generate code to call gl_malloc
    ///
    /// Input: rdi = size in bytes
    /// Output: rax = pointer to allocated memory (or NULL on failure)
    ///
    /// ```asm
    /// call gl_malloc
    /// ```
    pub fn gen_malloc_call() -> Vec<Instruction> {
        vec![
            Instruction::Comment("Allocate heap memory via gl_malloc".to_string()),
            Instruction::Call("gl_malloc".to_string()),
            Instruction::Comment("Allocated pointer now in rax".to_string()),
        ]
    }

    /// Generate code to call gl_free
    ///
    /// Input: rdi = pointer to free
    /// Output: none
    ///
    /// ```asm
    /// call gl_free
    /// ```
    pub fn gen_free_call() -> Vec<Instruction> {
        vec![
            Instruction::Comment("Free heap memory via gl_free".to_string()),
            Instruction::Call("gl_free".to_string()),
        ]
    }

    /// Generate code to allocate a struct
    ///
    /// Allocates heap memory for a struct with the given number of fields.
    /// Each field is 8 bytes (f64 or pointer).
    ///
    /// Input: field_count = number of fields
    /// Output: Returns instructions that leave pointer in rax
    pub fn gen_struct_alloc(field_count: usize) -> Vec<Instruction> {
        let size = field_count * 8;  // 8 bytes per field
        let mut code = Vec::new();

        code.push(Instruction::Comment(format!(
            "Allocate struct with {} fields ({} bytes)",
            field_count, size
        )));

        // Load size into rdi (first argument to malloc)
        code.push(Instruction::Mov(
            format!("${}", size),
            "%rdi".to_string()
        ));

        // Call malloc
        code.extend(Self::gen_malloc_call());

        // Check if malloc returned NULL
        code.push(Instruction::Comment("Check for allocation failure".to_string()));
        code.push(Instruction::Cmp(
            "$0".to_string(),
            "%rax".to_string()
        ));

        // TODO: Handle allocation failure (for now, assume success)
        // In production, should jump to error handler

        code
    }

    /// Generate code to allocate a string
    ///
    /// Allocates heap memory for a string with length prefix.
    ///
    /// Input: r10 = string length (in bytes)
    ///        r11 = pointer to string data (source)
    /// Output: rax = pointer to allocated string (with length prefix)
    pub fn gen_string_alloc() -> Vec<Instruction> {
        let mut code = vec![
            Instruction::Comment("Allocate string on heap".to_string()),
            // Calculate total size: 8 bytes (length) + string data
            Instruction::Mov(
                "%r10".to_string(),
                "%rdi".to_string()
            ),
            Instruction::Add(
                "$8".to_string(),
                "%rdi".to_string()
            ),
            // Save r10 and r11 (will be clobbered by malloc call)
            Instruction::Push("%r10".to_string()),
            Instruction::Push("%r11".to_string()),
        ];

        // Call malloc
        code.extend(Self::gen_malloc_call());

        // Restore r10 and r11
        code.push(Instruction::Pop("%r11".to_string()));
        code.push(Instruction::Pop("%r10".to_string()));

        // Store length at offset 0
        code.push(Instruction::Mov(
            "%r10".to_string(),
            "0(%rax)".to_string()
        ));

        // Copy string data byte-by-byte
        // rax = destination string (with length prefix)
        // r11 = source string data
        // r10 = length
        code.push(Instruction::Comment("Copy string data byte-by-byte".to_string()));

        // Initialize copy: rcx = counter (0), rdx = destination offset (8)
        code.push(Instruction::Xor("%rcx".to_string(), "%rcx".to_string())); // rcx = 0
        code.push(Instruction::Mov("$8".to_string(), "%rdx".to_string()));   // rdx = 8 (skip length)

        // Loop start label
        code.push(Instruction::Label(".L_string_copy_loop".to_string()));

        // Check if rcx >= r10 (copied all bytes?)
        code.push(Instruction::Cmp("%r10".to_string(), "%rcx".to_string()));
        code.push(Instruction::Jge(".L_string_copy_done".to_string()));

        // Copy one byte: byte = *(r11 + rcx)
        code.push(Instruction::Mov(
            "(%r11,%rcx,1)".to_string(),
            "%r8b".to_string()  // r8b = 8-bit register for byte
        ));

        // Store byte: *(rax + rdx) = byte
        code.push(Instruction::Mov(
            "%r8b".to_string(),
            "(%rax,%rdx,1)".to_string()
        ));

        // Increment counters
        code.push(Instruction::Inc("%rcx".to_string()));  // rcx++
        code.push(Instruction::Inc("%rdx".to_string()));  // rdx++

        // Loop back
        code.push(Instruction::Jmp(".L_string_copy_loop".to_string()));

        // Loop done
        code.push(Instruction::Label(".L_string_copy_done".to_string()));
        code.push(Instruction::Comment("String allocated at rax".to_string()));

        code
    }

    /// Generate code to free a struct
    ///
    /// Input: rax = pointer to struct
    pub fn gen_struct_free() -> Vec<Instruction> {
        let mut code = Vec::new();

        code.push(Instruction::Comment("Free struct memory".to_string()));

        // Move pointer to rdi (first argument to free)
        code.push(Instruction::Mov(
            "%rax".to_string(),
            "%rdi".to_string()
        ));

        // Call free
        code.extend(Self::gen_free_call());

        code
    }

    /// Generate code to load a struct field
    ///
    /// Input: rax = struct pointer
    ///        field_index = index of field to load
    /// Output: rax = field value
    pub fn gen_struct_field_load(field_index: usize) -> Vec<Instruction> {
        let offset = field_index * 8;  // 8 bytes per field
        let mut code = Vec::new();

        code.push(Instruction::Comment(format!(
            "Load struct field {} (offset {})",
            field_index, offset
        )));

        code.push(Instruction::Mov(
            format!("{}(%rax)", offset),
            "%rax".to_string()
        ));

        code
    }

    /// Generate code to store a value to a struct field
    ///
    /// Input: rbx = struct pointer
    ///        rax = value to store
    ///        field_index = index of field to store
    pub fn gen_struct_field_store(field_index: usize) -> Vec<Instruction> {
        let offset = field_index * 8;
        let mut code = Vec::new();

        code.push(Instruction::Comment(format!(
            "Store to struct field {} (offset {})",
            field_index, offset
        )));

        code.push(Instruction::Mov(
            "%rax".to_string(),
            format!("{}(%rbx)", offset)
        ));

        code
    }

    /// Generate external function declarations
    ///
    /// Declares gl_malloc and gl_free as external functions that will be
    /// provided by linking with native_allocator.S.
    pub fn gen_external_declarations() -> String {
        "    # External runtime functions (custom allocator in native_allocator.S)\n\
         .globl gl_malloc\n\
         .globl gl_free\n\n".to_string()
    }
}

impl Default for NativeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_malloc_call() {
        let code = NativeRuntime::gen_malloc_call();
        assert!(code.len() >= 1);

        // Should contain call to gl_malloc
        assert!(code.iter().any(|inst| {
            matches!(inst, Instruction::Call(name) if name == "gl_malloc")
        }));
    }

    #[test]
    fn test_gen_struct_alloc() {
        let code = NativeRuntime::gen_struct_alloc(3);

        // Should calculate size (3 fields * 8 bytes = 24)
        assert!(code.iter().any(|inst| {
            matches!(inst, Instruction::Mov(src, dst)
                if src == "$24" && dst == "%rdi")
        }));

        // Should call gl_malloc
        assert!(code.iter().any(|inst| {
            matches!(inst, Instruction::Call(name) if name == "gl_malloc")
        }));
    }

    #[test]
    fn test_gen_struct_field_load() {
        let code = NativeRuntime::gen_struct_field_load(2);

        // Should load from offset 16 (field 2 * 8 bytes)
        assert!(code.iter().any(|inst| {
            matches!(inst, Instruction::Mov(src, dst)
                if src == "16(%rax)" && dst == "%rax")
        }));
    }

    #[test]
    fn test_gen_struct_field_store() {
        let code = NativeRuntime::gen_struct_field_store(1);

        // Should store to offset 8 (field 1 * 8 bytes)
        assert!(code.iter().any(|inst| {
            matches!(inst, Instruction::Mov(src, dst)
                if src == "%rax" && dst == "8(%rbx)")
        }));
    }

    #[test]
    fn test_gen_external_declarations() {
        let decls = NativeRuntime::gen_external_declarations();

        assert!(decls.contains("gl_malloc"));
        assert!(decls.contains("gl_free"));
        assert!(decls.contains(".globl"));
    }
}
