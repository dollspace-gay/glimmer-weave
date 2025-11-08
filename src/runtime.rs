//! # Glimmer-Weave Runtime Library
//!
//! Standard library functions and utilities for Glimmer-Weave programs.
//!
//! This module provides builtin functions for:
//! - String manipulation (length, slice, concat, upper, lower, split, join, trim, replace, repeat, pad, reverse)
//! - Math operations (abs, sqrt, pow, min, max, floor, ceil, round, sign, clamp, sin, cos, tan, log, exp)
//! - List operations (length, push, pop, reverse, concat, slice, flatten, sum, product, min, max, contains)
//! - Map operations (keys, values, has, size)
//! - Type conversion (to_text, to_number, to_truth, type_of)
//! - Outcome/Maybe helpers (is_triumph, expect_present, refine_triumph, etc.)
//! - Iterator operations (iter, iter_next, iter_map, iter_filter, iter_fold, iter_collect, iter_take)
//! - I/O operations (print, println - require kernel context)

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::boxed::Box;
use crate::eval::{Value, RuntimeError};

/// Math functions abstraction - use std when available (tests), libm when no_std
mod math {
    // Use std math functions when std is available (includes tests)
    #[cfg(feature = "std")]
    pub fn sqrt(x: f64) -> f64 { x.sqrt() }

    #[cfg(feature = "std")]
    pub fn pow(x: f64, y: f64) -> f64 { x.powf(y) }

    #[cfg(feature = "std")]
    pub fn floor(x: f64) -> f64 { x.floor() }

    #[cfg(feature = "std")]
    pub fn ceil(x: f64) -> f64 { x.ceil() }

    #[cfg(feature = "std")]
    pub fn round(x: f64) -> f64 { x.round() }

    #[cfg(feature = "std")]
    pub fn sin(x: f64) -> f64 { x.sin() }

    #[cfg(feature = "std")]
    pub fn cos(x: f64) -> f64 { x.cos() }

    #[cfg(feature = "std")]
    pub fn tan(x: f64) -> f64 { x.tan() }

    #[cfg(feature = "std")]
    pub fn log(x: f64) -> f64 { x.ln() }

    #[cfg(feature = "std")]
    pub fn exp(x: f64) -> f64 { x.exp() }

    // Use libm when std is not available (no_std mode)
    #[cfg(not(feature = "std"))]
    pub use libm::{sqrt, pow, floor, ceil, round, sin, cos, tan, log, exp};
}

/// Type signature for native function implementations
pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

/// Native function wrapper with name and implementation
#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub func: NativeFn,
    pub arity: Option<usize>,  // None = variadic
}

impl NativeFunction {
    pub fn new(name: &str, arity: Option<usize>, func: NativeFn) -> Self {
        NativeFunction {
            name: name.to_string(),
            arity,
            func,
        }
    }
}

impl core::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Get all builtin functions
pub fn get_builtins() -> Vec<NativeFunction> {
    vec![
        // === String Functions ===
        NativeFunction::new("length", Some(1), string_length),
        NativeFunction::new("slice", Some(3), string_slice),
        NativeFunction::new("concat", Some(2), string_concat),
        NativeFunction::new("upper", Some(1), string_upper),
        NativeFunction::new("lower", Some(1), string_lower),
        NativeFunction::new("split", Some(2), string_split),
        NativeFunction::new("join", Some(2), string_join),
        NativeFunction::new("trim", Some(1), string_trim),
        NativeFunction::new("starts_with", Some(2), string_starts_with),
        NativeFunction::new("ends_with", Some(2), string_ends_with),
        NativeFunction::new("contains", Some(2), string_contains),
        NativeFunction::new("replace", Some(3), string_replace),
        NativeFunction::new("char_at", Some(2), string_char_at),
        NativeFunction::new("repeat", Some(2), string_repeat),
        NativeFunction::new("pad_left", Some(3), string_pad_left),
        NativeFunction::new("pad_right", Some(3), string_pad_right),
        NativeFunction::new("reverse", Some(1), string_reverse),

        // === Math Functions ===
        NativeFunction::new("abs", Some(1), math_abs),
        NativeFunction::new("sqrt", Some(1), math_sqrt),
        NativeFunction::new("pow", Some(2), math_pow),
        NativeFunction::new("min", Some(2), math_min),
        NativeFunction::new("max", Some(2), math_max),
        NativeFunction::new("floor", Some(1), math_floor),
        NativeFunction::new("ceil", Some(1), math_ceil),
        NativeFunction::new("round", Some(1), math_round),
        NativeFunction::new("sign", Some(1), math_sign),
        NativeFunction::new("clamp", Some(3), math_clamp),
        NativeFunction::new("sin", Some(1), math_sin),
        NativeFunction::new("cos", Some(1), math_cos),
        NativeFunction::new("tan", Some(1), math_tan),
        NativeFunction::new("log", Some(1), math_log),
        NativeFunction::new("exp", Some(1), math_exp),

        // === List Functions ===
        NativeFunction::new("list_length", Some(1), list_length),
        NativeFunction::new("list_push", Some(2), list_push),
        NativeFunction::new("list_pop", Some(1), list_pop),
        NativeFunction::new("list_reverse", Some(1), list_reverse),
        NativeFunction::new("list_first", Some(1), list_first),
        NativeFunction::new("list_last", Some(1), list_last),
        NativeFunction::new("list_concat", Some(2), list_concat),
        NativeFunction::new("list_slice", Some(3), list_slice),
        NativeFunction::new("list_flatten", Some(1), list_flatten),
        NativeFunction::new("list_sum", Some(1), list_sum),
        NativeFunction::new("list_product", Some(1), list_product),
        NativeFunction::new("list_min", Some(1), list_min),
        NativeFunction::new("list_max", Some(1), list_max),
        NativeFunction::new("list_contains", Some(2), list_contains),
        NativeFunction::new("list_index_of", Some(2), list_index_of),

        // === Map Functions ===
        NativeFunction::new("map_keys", Some(1), map_keys),
        NativeFunction::new("map_values", Some(1), map_values),
        NativeFunction::new("map_has", Some(2), map_has),
        NativeFunction::new("map_size", Some(1), map_size),

        // === Type Conversion ===
        NativeFunction::new("to_text", Some(1), to_text),
        NativeFunction::new("to_number", Some(1), to_number),
        NativeFunction::new("to_truth", Some(1), to_truth),
        NativeFunction::new("type_of", Some(1), type_of),

        // === I/O Functions ===
        NativeFunction::new("print", None, io_print),
        NativeFunction::new("println", None, io_println),

        // === Outcome<T, E> Helper Functions ===
        // Inspection
        NativeFunction::new("is_triumph", Some(1), is_triumph),
        NativeFunction::new("is_mishap", Some(1), is_mishap),

        // Extraction
        NativeFunction::new("expect_triumph", Some(2), expect_triumph),
        NativeFunction::new("triumph_or", Some(2), triumph_or),
        NativeFunction::new("triumph_or_else", Some(2), triumph_or_else),
        NativeFunction::new("expect_mishap", Some(2), expect_mishap),

        // Transformation
        NativeFunction::new("refine_triumph", Some(2), refine_triumph),
        NativeFunction::new("refine_mishap", Some(2), refine_mishap),

        // Chaining
        NativeFunction::new("then_triumph", Some(2), then_triumph),

        // === Maybe<T> Helper Functions ===
        // Inspection
        NativeFunction::new("is_present", Some(1), is_present),
        NativeFunction::new("is_absent", Some(1), is_absent),

        // Extraction
        NativeFunction::new("expect_present", Some(2), expect_present),
        NativeFunction::new("present_or", Some(2), present_or),
        NativeFunction::new("present_or_else", Some(2), present_or_else),

        // Transformation
        NativeFunction::new("refine_present", Some(2), refine_present),

        // Chaining
        NativeFunction::new("then_present", Some(2), then_present),

        // === Conversion Functions ===
        NativeFunction::new("present_or_mishap", Some(2), present_or_mishap),
        NativeFunction::new("triumph_or_absent", Some(1), triumph_or_absent),

        // === Combination Functions ===
        NativeFunction::new("both_triumph", Some(2), both_triumph),
        NativeFunction::new("either_triumph", Some(2), either_triumph),

        // === Enum (Variant) Helper Functions - Phase 4 ===
        // Inspection
        NativeFunction::new("is_variant", Some(2), is_variant),

        // Extraction
        NativeFunction::new("expect_variant", Some(3), expect_variant),
        NativeFunction::new("variant_or", Some(3), variant_or),

        // Transformation
        NativeFunction::new("refine_variant", Some(3), refine_variant),

        // === Iterator Functions - Phase 1 ===
        // Core iteration
        NativeFunction::new("iter", Some(1), iter_create),
        NativeFunction::new("iter_next", Some(1), iter_next),

        // Transformation
        NativeFunction::new("iter_map", Some(2), iter_map),
        NativeFunction::new("iter_filter", Some(2), iter_filter),

        // Reduction
        NativeFunction::new("iter_fold", Some(3), iter_fold),
        NativeFunction::new("iter_collect", Some(1), iter_collect),

        // Limiting
        NativeFunction::new("iter_take", Some(2), iter_take),

        // === Smart Pointer Functions ===
        // Shared<T> (Rc-like) operations
        NativeFunction::new("Shared_new", Some(1), shared_new),
        NativeFunction::new("Shared_get", Some(1), shared_get),
        NativeFunction::new("Shared_clone", Some(1), shared_clone),
        NativeFunction::new("Shared_count", Some(1), shared_count),

        // Cell<T> (RefCell-like) operations
        NativeFunction::new("Cell_new", Some(1), cell_new),
        NativeFunction::new("Cell_get", Some(1), cell_get),
        NativeFunction::new("Cell_set", Some(2), cell_set),
        NativeFunction::new("Cell_borrow", Some(1), cell_borrow),
        NativeFunction::new("Cell_borrow_mut", Some(1), cell_borrow_mut),
        NativeFunction::new("Cell_release", Some(1), cell_release),
    ]
}

// ============================================================================
// STRING FUNCTIONS
// ============================================================================

fn string_length(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Text(s) => Ok(Value::Number(s.len() as f64)),
        v => Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn string_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Text(s), Value::Number(start), Value::Number(end)) => {
            let start = *start as usize;
            let end = *end as usize;

            if start > s.len() || end > s.len() || start > end {
                return Err(RuntimeError::IndexOutOfBounds {
                    index: if start > end { start } else { end },
                    length: s.len(),
                });
            }

            Ok(Value::Text(s[start..end].to_string()))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Number, Number".to_string(),
            got: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()),
        }),
    }
}

fn string_concat(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s1), Value::Text(s2)) => {
            let mut result = s1.clone();
            result.push_str(s2);
            Ok(Value::Text(result))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_upper(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Text(s) => {
            let mut result = String::new();
            for c in s.chars() {
                result.push(c.to_ascii_uppercase());
            }
            Ok(Value::Text(result))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn string_lower(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Text(s) => {
            let mut result = String::new();
            for c in s.chars() {
                result.push(c.to_ascii_lowercase());
            }
            Ok(Value::Text(result))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn string_split(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(delimiter)) => {
            let parts: Vec<Value> = s.split(delimiter.as_str())
                .map(|part| Value::Text(part.to_string()))
                .collect();
            Ok(Value::List(parts))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_join(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::List(items), Value::Text(separator)) => {
            let strings: Result<Vec<String>, RuntimeError> = items.iter()
                .map(|v| match v {
                    Value::Text(s) => Ok(s.clone()),
                    v => Err(RuntimeError::TypeError {
                        expected: "Text".to_string(),
                        got: v.type_name().to_string(),
                    }),
                })
                .collect();

            let strings = strings?;
            let mut result = String::new();
            for (i, s) in strings.iter().enumerate() {
                if i > 0 {
                    result.push_str(separator);
                }
                result.push_str(s);
            }
            Ok(Value::Text(result))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "List, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_trim(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Text(s) => Ok(Value::Text(s.trim().to_string())),
        v => Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn string_starts_with(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(prefix)) => {
            Ok(Value::Truth(s.starts_with(prefix.as_str())))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_ends_with(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(suffix)) => {
            Ok(Value::Truth(s.ends_with(suffix.as_str())))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Text(substring)) => {
            Ok(Value::Truth(s.contains(substring.as_str())))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Text(s), Value::Text(from), Value::Text(to)) => {
            Ok(Value::Text(s.replace(from.as_str(), to.as_str())))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Text, Text".to_string(),
            got: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()),
        }),
    }
}

fn string_char_at(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Number(index)) => {
            let index = *index as usize;
            if index >= s.len() {
                return Err(RuntimeError::IndexOutOfBounds {
                    index,
                    length: s.len(),
                });
            }
            let ch = s.chars().nth(index).ok_or(RuntimeError::IndexOutOfBounds {
                index,
                length: s.len(),
            })?;
            Ok(Value::Text(ch.to_string()))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Number".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_repeat(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Text(s), Value::Number(n)) => {
            let n = *n as usize;
            let mut result = String::new();
            for _ in 0..n {
                result.push_str(s);
            }
            Ok(Value::Text(result))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Number".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn string_pad_left(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Text(s), Value::Number(width), Value::Text(pad_char)) => {
            let width = *width as usize;
            if pad_char.len() != 1 {
                return Err(RuntimeError::Custom("Pad character must be a single character".to_string()));
            }
            let pad_ch = pad_char.chars().next().unwrap();

            if s.len() >= width {
                Ok(Value::Text(s.clone()))
            } else {
                let mut result = String::new();
                for _ in 0..(width - s.len()) {
                    result.push(pad_ch);
                }
                result.push_str(s);
                Ok(Value::Text(result))
            }
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Number, Text".to_string(),
            got: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()),
        }),
    }
}

fn string_pad_right(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Text(s), Value::Number(width), Value::Text(pad_char)) => {
            let width = *width as usize;
            if pad_char.len() != 1 {
                return Err(RuntimeError::Custom("Pad character must be a single character".to_string()));
            }
            let pad_ch = pad_char.chars().next().unwrap();

            if s.len() >= width {
                Ok(Value::Text(s.clone()))
            } else {
                let mut result = s.clone();
                for _ in 0..(width - s.len()) {
                    result.push(pad_ch);
                }
                Ok(Value::Text(result))
            }
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Text, Number, Text".to_string(),
            got: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()),
        }),
    }
}

fn string_reverse(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Text(s) => {
            let reversed: String = s.chars().rev().collect();
            Ok(Value::Text(reversed))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// MATH FUNCTIONS
// ============================================================================

fn math_abs(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_sqrt(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                Err(RuntimeError::Custom("Cannot take square root of negative number".to_string()))
            } else {
                Ok(Value::Number(math::sqrt(*n)))
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_pow(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => {
            Ok(Value::Number(math::pow(*base, *exp)))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Number, Number".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn math_min(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(if a < b { *a } else { *b }))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Number, Number".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn math_max(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(if a > b { *a } else { *b }))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Number, Number".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn math_floor(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::floor(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_ceil(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::ceil(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_round(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::round(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_sign(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => {
            let sign = if *n > 0.0 {
                1.0
            } else if *n < 0.0 {
                -1.0
            } else {
                0.0
            };
            Ok(Value::Number(sign))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_clamp(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Number(value), Value::Number(min_val), Value::Number(max_val)) => {
            if min_val > max_val {
                return Err(RuntimeError::Custom("Clamp min must be <= max".to_string()));
            }
            let clamped = if *value < *min_val {
                *min_val
            } else if *value > *max_val {
                *max_val
            } else {
                *value
            };
            Ok(Value::Number(clamped))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Number, Number, Number".to_string(),
            got: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()),
        }),
    }
}

fn math_sin(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::sin(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_cos(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::cos(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_tan(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::tan(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_log(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => {
            if *n <= 0.0 {
                Err(RuntimeError::Custom("Cannot take log of non-positive number".to_string()))
            } else {
                Ok(Value::Number(math::log(*n)))
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn math_exp(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(math::exp(*n))),
        v => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// LIST FUNCTIONS
// ============================================================================

fn list_length(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => Ok(Value::Number(l.len() as f64)),
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_push(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let mut new_list = l.clone();
            new_list.push(args[1].clone());
            Ok(Value::List(new_list))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                return Err(RuntimeError::Custom("Cannot pop from empty list".to_string()));
            }
            let mut new_list = l.clone();
            new_list.pop();
            Ok(Value::List(new_list))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_reverse(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let mut new_list = l.clone();
            new_list.reverse();
            Ok(Value::List(new_list))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_first(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                return Err(RuntimeError::Custom("Cannot get first element of empty list".to_string()));
            }
            Ok(l[0].clone())
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_last(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                return Err(RuntimeError::Custom("Cannot get last element of empty list".to_string()));
            }
            Ok(l[l.len() - 1].clone())
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_concat(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::List(l1), Value::List(l2)) => {
            let mut result = l1.clone();
            result.extend(l2.clone());
            Ok(Value::List(result))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "List, List".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn list_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::List(l), Value::Number(start), Value::Number(end)) => {
            let start = *start as usize;
            let end = *end as usize;

            if start > l.len() || end > l.len() || start > end {
                return Err(RuntimeError::IndexOutOfBounds {
                    index: if start > end { start } else { end },
                    length: l.len(),
                });
            }

            Ok(Value::List(l[start..end].to_vec()))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "List, Number, Number".to_string(),
            got: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()),
        }),
    }
}

fn list_flatten(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let mut result = Vec::new();
            for item in l.iter() {
                match item {
                    Value::List(inner) => {
                        result.extend(inner.clone());
                    }
                    other => {
                        result.push(other.clone());
                    }
                }
            }
            Ok(Value::List(result))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_sum(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let mut sum = 0.0;
            for item in l.iter() {
                match item {
                    Value::Number(n) => sum += n,
                    v => return Err(RuntimeError::TypeError {
                        expected: "Number".to_string(),
                        got: v.type_name().to_string(),
                    }),
                }
            }
            Ok(Value::Number(sum))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_product(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let mut product = 1.0;
            for item in l.iter() {
                match item {
                    Value::Number(n) => product *= n,
                    v => return Err(RuntimeError::TypeError {
                        expected: "Number".to_string(),
                        got: v.type_name().to_string(),
                    }),
                }
            }
            Ok(Value::Number(product))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_min(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                return Err(RuntimeError::Custom("Cannot find min of empty list".to_string()));
            }

            let mut min_val = match &l[0] {
                Value::Number(n) => *n,
                v => return Err(RuntimeError::TypeError {
                    expected: "Number".to_string(),
                    got: v.type_name().to_string(),
                }),
            };

            for item in l.iter().skip(1) {
                match item {
                    Value::Number(n) => {
                        if *n < min_val {
                            min_val = *n;
                        }
                    }
                    v => return Err(RuntimeError::TypeError {
                        expected: "Number".to_string(),
                        got: v.type_name().to_string(),
                    }),
                }
            }
            Ok(Value::Number(min_val))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_max(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            if l.is_empty() {
                return Err(RuntimeError::Custom("Cannot find max of empty list".to_string()));
            }

            let mut max_val = match &l[0] {
                Value::Number(n) => *n,
                v => return Err(RuntimeError::TypeError {
                    expected: "Number".to_string(),
                    got: v.type_name().to_string(),
                }),
            };

            for item in l.iter().skip(1) {
                match item {
                    Value::Number(n) => {
                        if *n > max_val {
                            max_val = *n;
                        }
                    }
                    v => return Err(RuntimeError::TypeError {
                        expected: "Number".to_string(),
                        got: v.type_name().to_string(),
                    }),
                }
            }
            Ok(Value::Number(max_val))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_contains(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let target = &args[1];
            for item in l.iter() {
                // Simple value equality check
                let matches = match (item, target) {
                    (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
                    (Value::Text(a), Value::Text(b)) => a == b,
                    (Value::Truth(a), Value::Truth(b)) => a == b,
                    (Value::Nothing, Value::Nothing) => true,
                    _ => false,
                };
                if matches {
                    return Ok(Value::Truth(true));
                }
            }
            Ok(Value::Truth(false))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn list_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::List(l) => {
            let target = &args[1];
            for (i, item) in l.iter().enumerate() {
                // Simple value equality check
                let matches = match (item, target) {
                    (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
                    (Value::Text(a), Value::Text(b)) => a == b,
                    (Value::Truth(a), Value::Truth(b)) => a == b,
                    (Value::Nothing, Value::Nothing) => true,
                    _ => false,
                };
                if matches {
                    return Ok(Value::Number(i as f64));
                }
            }
            // Return -1 if not found (Glimmer-Weave convention)
            Ok(Value::Number(-1.0))
        }
        v => Err(RuntimeError::TypeError {
            expected: "List".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// MAP FUNCTIONS
// ============================================================================

fn map_keys(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Map(m) => {
            let keys: Vec<Value> = m.keys()
                .map(|k| Value::Text(k.clone()))
                .collect();
            Ok(Value::List(keys))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Map".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn map_values(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Map(m) => {
            let values: Vec<Value> = m.values()
                .cloned()
                .collect();
            Ok(Value::List(values))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Map".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn map_has(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Map(m), Value::Text(key)) => {
            Ok(Value::Truth(m.contains_key(key)))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Map, Text".to_string(),
            got: format!("{}, {}", args[0].type_name(), args[1].type_name()),
        }),
    }
}

fn map_size(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Map(m) => Ok(Value::Number(m.len() as f64)),
        v => Err(RuntimeError::TypeError {
            expected: "Map".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// TYPE CONVERSION FUNCTIONS
// ============================================================================

fn to_text(args: &[Value]) -> Result<Value, RuntimeError> {
    let text = match &args[0] {
        Value::Number(n) => format!("{}", n),
        Value::Text(s) => s.clone(),
        Value::Truth(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::Nothing => "nothing".to_string(),
        Value::List(_) => "[List]".to_string(),
        Value::Map(_) => "[Map]".to_string(),
        Value::Chant { .. } => "[Chant]".to_string(),
        Value::NativeChant(native_fn) => format!("[NativeChant:{}]", native_fn.name),
        Value::Capability { .. } => "[Capability]".to_string(),
        Value::Range { .. } => "[Range]".to_string(),
        Value::Outcome { success, value } => {
            // Recursively convert inner value to text
            let inner_text = to_text(&[*value.clone()])?;
            if let Value::Text(inner) = inner_text {
                if *success {
                    format!("Triumph({})", inner)
                } else {
                    format!("Mishap({})", inner)
                }
            } else {
                unreachable!("to_text always returns Text")
            }
        }
        Value::Maybe { present, value } => {
            if *present {
                if let Some(v) = value {
                    let inner_text = to_text(&[*v.clone()])?;
                    if let Value::Text(inner) = inner_text {
                        format!("Present({})", inner)
                    } else {
                        unreachable!("to_text always returns Text")
                    }
                } else {
                    "Present(nothing)".to_string()
                }
            } else {
                "Absent".to_string()
            }
        }
        Value::StructDef { name, .. } => {
            format!("[StructDef:{}]", name)
        }
        Value::StructInstance { struct_name, fields } => {
            // Format as StructName { field1: value1, field2: value2 }
            let mut field_strings = Vec::new();
            for (k, v) in fields.iter() {
                let v_text = to_text(core::slice::from_ref(v))?;
                if let Value::Text(s) = v_text {
                    field_strings.push(format!("{}: {}", k, s));
                } else {
                    unreachable!("to_text always returns Text")
                }
            }
            format!("{} {{ {} }}", struct_name, field_strings.join(", "))
        }
        Value::VariantDef { name, .. } => {
            format!("[VariantDef:{}]", name)
        }
        Value::VariantValue { variant_name, fields, .. } => {
            // Phase 1: Simple enums (no fields) - just show variant name
            // Phase 2: With fields - show as VariantName(field1, field2)
            if fields.is_empty() {
                variant_name.clone()
            } else {
                // Phase 2: Format fields
                let mut field_strings = Vec::new();
                for v in fields.iter() {
                    let v_text = to_text(core::slice::from_ref(v))?;
                    if let Value::Text(s) = v_text {
                        field_strings.push(s);
                    } else {
                        unreachable!("to_text always returns Text")
                    }
                }
                format!("{}({})", variant_name, field_strings.join(", "))
            }
        }
        Value::VariantConstructor { variant_name, .. } => {
            // Phase 2: Show constructor as a callable function
            format!("[VariantConstructor:{}]", variant_name)
        }
        Value::Iterator { iterator_type, .. } => {
            format!("[Iterator:{}]", iterator_type)
        }
        Value::Shared { value, ref_count } => {
            // Show Shared with ref count and inner value type
            format!("[Shared<{}> (refs: {})]", value.type_name(), ref_count)
        }
        Value::Cell { value, .. } => {
            // Show Cell with inner value type
            format!("[Cell<{}>]", value.type_name())
        }
    };
    Ok(Value::Text(text))
}

fn to_number(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::Text(s) => {
            s.parse::<f64>()
                .map(Value::Number)
                .map_err(|_| RuntimeError::Custom(format!("Cannot convert '{}' to number", s)))
        }
        Value::Truth(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        v => Err(RuntimeError::TypeError {
            expected: "Number, Text, or Truth".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

fn to_truth(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Truth(args[0].is_truthy()))
}

fn type_of(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Text(args[0].type_name().to_string()))
}

// ============================================================================
// I/O FUNCTIONS
// ============================================================================
//
// NOTE: These are stub implementations. Real I/O should be provided by
// the host environment (kernel) via capability-based syscalls.
// For now, these functions are not implemented and will return errors.

fn io_print(_args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::Custom(
        "print() requires kernel I/O capabilities - call from kernel context only".to_string()
    ))
}

fn io_println(_args: &[Value]) -> Result<Value, RuntimeError> {
    Err(RuntimeError::Custom(
        "println() requires kernel I/O capabilities - call from kernel context only".to_string()
    ))
}

// ============================================================================
// OUTCOME<T, E> HELPER FUNCTIONS
// ============================================================================

/// Check if an Outcome is Triumph (success)
fn is_triumph(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success, .. } => Ok(Value::Truth(*success)),
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Check if an Outcome is Mishap (failure)
fn is_mishap(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success, .. } => Ok(Value::Truth(!*success)),
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get triumph value or panic with custom message
fn expect_triumph(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Outcome { success: true, value }, _) => Ok(*value.clone()),
        (Value::Outcome { success: false, .. }, Value::Text(msg)) => {
            Err(RuntimeError::Custom(msg.clone()))
        }
        (Value::Outcome { success: false, .. }, _) => {
            Err(RuntimeError::Custom("expect_triumph failed".to_string()))
        }
        (v, _) => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get triumph value or return default
fn triumph_or(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success: true, value } => Ok(*value.clone()),
        Value::Outcome { success: false, .. } => Ok(args[1].clone()),
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get triumph value or compute default using function
fn triumph_or_else(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success: true, value } => Ok(*value.clone()),
        Value::Outcome { success: false, .. } => {
            // Call the function with no arguments
            match &args[1] {
                Value::Chant { params: _params, body: _body, closure: _ } => {
                    if !_params.is_empty() {
                        return Err(RuntimeError::ArityMismatch {
                            expected: 0,
                            got: _params.len(),
                        });
                    }
                    // We need to create an evaluator to call the function
                    // For now, return the function itself
                    // TODO: Need better support for calling functions from native code
                    Err(RuntimeError::Custom(
                        "triumph_or_else: Function execution not yet supported from native code".to_string()
                    ))
                }
                v => Err(RuntimeError::TypeError {
                    expected: "Chant".to_string(),
                    got: v.type_name().to_string(),
                }),
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get mishap value or panic with custom message
fn expect_mishap(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Outcome { success: false, value }, _) => Ok(*value.clone()),
        (Value::Outcome { success: true, .. }, Value::Text(msg)) => {
            Err(RuntimeError::Custom(msg.clone()))
        }
        (Value::Outcome { success: true, .. }, _) => {
            Err(RuntimeError::Custom("expect_mishap failed".to_string()))
        }
        (v, _) => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Transform triumph value (map operation)
fn refine_triumph(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success: true, value: _value } => {
            // Need to apply function to value
            // For now, return error as we need evaluator context
            Err(RuntimeError::Custom(
                "refine_triumph: Function execution not yet supported from native code".to_string()
            ))
        }
        Value::Outcome { success: false, value } => {
            // Return mishap unchanged
            Ok(Value::Outcome {
                success: false,
                value: value.clone(),
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Transform mishap value
fn refine_mishap(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success: false, value: _value } => {
            // Need to apply function to error value
            Err(RuntimeError::Custom(
                "refine_mishap: Function execution not yet supported from native code".to_string()
            ))
        }
        Value::Outcome { success: true, value } => {
            // Return triumph unchanged
            Ok(Value::Outcome {
                success: true,
                value: value.clone(),
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Chain outcomes (flatMap operation)
fn then_triumph(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success: true, value: _ } => {
            // Need to apply function and flatten result
            Err(RuntimeError::Custom(
                "then_triumph: Function execution not yet supported from native code".to_string()
            ))
        }
        Value::Outcome { success: false, value } => {
            // Return mishap unchanged
            Ok(Value::Outcome {
                success: false,
                value: value.clone(),
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// MAYBE<T> HELPER FUNCTIONS
// ============================================================================

/// Check if a Maybe is Present
fn is_present(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present, .. } => Ok(Value::Truth(*present)),
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Check if a Maybe is Absent
fn is_absent(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present, .. } => Ok(Value::Truth(!*present)),
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get present value or panic with custom message
fn expect_present(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Maybe { present: true, value: Some(v) }, _) => Ok(*v.clone()),
        (Value::Maybe { present: false, .. }, Value::Text(msg)) => {
            Err(RuntimeError::Custom(msg.clone()))
        }
        (Value::Maybe { present: false, .. }, _) => {
            Err(RuntimeError::Custom("expect_present failed".to_string()))
        }
        (v, _) => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get present value or return default
fn present_or(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present: true, value: Some(v) } => Ok(*v.clone()),
        Value::Maybe { present: false, .. } => Ok(args[1].clone()),
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get present value or compute default using function
fn present_or_else(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present: true, value: Some(v) } => Ok(*v.clone()),
        Value::Maybe { present: false, .. } => {
            // Call the function with no arguments
            Err(RuntimeError::Custom(
                "present_or_else: Function execution not yet supported from native code".to_string()
            ))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Transform present value (map operation)
fn refine_present(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present: true, value: Some(_v) } => {
            // Need to apply function to value
            Err(RuntimeError::Custom(
                "refine_present: Function execution not yet supported from native code".to_string()
            ))
        }
        Value::Maybe { present: false, value: None } => {
            // Return Absent unchanged
            Ok(Value::Maybe {
                present: false,
                value: None,
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Chain maybes (flatMap operation)
fn then_present(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present: true, value: Some(_v) } => {
            // Need to apply function and flatten result
            Err(RuntimeError::Custom(
                "then_present: Function execution not yet supported from native code".to_string()
            ))
        }
        Value::Maybe { present: false, value: None } => {
            // Return Absent unchanged
            Ok(Value::Maybe {
                present: false,
                value: None,
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// CONVERSION FUNCTIONS
// ============================================================================

/// Convert Maybe<T> to Outcome<T, E>
fn present_or_mishap(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Maybe { present: true, value: Some(v) } => {
            Ok(Value::Outcome {
                success: true,
                value: v.clone(),
            })
        }
        Value::Maybe { present: false, .. } => {
            Ok(Value::Outcome {
                success: false,
                value: Box::new(args[1].clone()),
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Maybe".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Convert Outcome<T, E> to Maybe<T> (discards error)
fn triumph_or_absent(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Outcome { success: true, value } => {
            Ok(Value::Maybe {
                present: true,
                value: Some(value.clone()),
            })
        }
        Value::Outcome { success: false, .. } => {
            Ok(Value::Maybe {
                present: false,
                value: None,
            })
        }
        v => Err(RuntimeError::TypeError {
            expected: "Outcome".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// COMBINATION FUNCTIONS
// ============================================================================

/// Combine two outcomes - both must be Triumph
fn both_triumph(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (
            Value::Outcome { success: true, value: v1 },
            Value::Outcome { success: true, value: v2 }
        ) => {
            // Create a list with both values (Pair representation)
            Ok(Value::Outcome {
                success: true,
                value: Box::new(Value::List(vec![*v1.clone(), *v2.clone()])),
            })
        }
        (Value::Outcome { success: false, value }, _) => {
            // First is mishap, return it
            Ok(Value::Outcome {
                success: false,
                value: value.clone(),
            })
        }
        (_, Value::Outcome { success: false, value }) => {
            // Second is mishap, return it
            Ok(Value::Outcome {
                success: false,
                value: value.clone(),
            })
        }
        (v1, v2) => {
            // Type error
            if !matches!(v1, Value::Outcome { .. }) {
                Err(RuntimeError::TypeError {
                    expected: "Outcome".to_string(),
                    got: v1.type_name().to_string(),
                })
            } else {
                Err(RuntimeError::TypeError {
                    expected: "Outcome".to_string(),
                    got: v2.type_name().to_string(),
                })
            }
        }
    }
}

/// Try first outcome, fallback to second on mishap
fn either_triumph(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Outcome { success: true, value }, _) => {
            // First is triumph, return it
            Ok(Value::Outcome {
                success: true,
                value: value.clone(),
            })
        }
        (Value::Outcome { success: false, .. }, Value::Outcome { success, value }) => {
            // First is mishap, return second
            Ok(Value::Outcome {
                success: *success,
                value: value.clone(),
            })
        }
        (v1, v2) => {
            // Type error
            if !matches!(v1, Value::Outcome { .. }) {
                Err(RuntimeError::TypeError {
                    expected: "Outcome".to_string(),
                    got: v1.type_name().to_string(),
                })
            } else {
                Err(RuntimeError::TypeError {
                    expected: "Outcome".to_string(),
                    got: v2.type_name().to_string(),
                })
            }
        }
    }
}

// ============================================================================
// ENUM (VARIANT) HELPER FUNCTIONS - Phase 4
// ============================================================================

/// Check if a value matches a specific variant
/// Usage: is_variant(enum_value, "VariantName") -> Truth
fn is_variant(args: &[Value]) -> Result<Value, RuntimeError> {
    let variant_name_to_check = match &args[1] {
        Value::Text(s) => s,
        v => return Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    };

    match &args[0] {
        Value::VariantValue { variant_name, .. } => {
            Ok(Value::Truth(variant_name == variant_name_to_check))
        }
        v => Err(RuntimeError::TypeError {
            expected: "VariantValue".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Extract data from a variant or panic with a message
/// Usage: expect_variant(enum_value, "VariantName", "error message") -> fields
fn expect_variant(args: &[Value]) -> Result<Value, RuntimeError> {
    let variant_name_to_check = match &args[1] {
        Value::Text(s) => s,
        v => return Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    };

    let error_message = match &args[2] {
        Value::Text(s) => s,
        v => return Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    };

    match &args[0] {
        Value::VariantValue { variant_name, fields, .. } => {
            if variant_name == variant_name_to_check {
                // Return the fields as a list
                Ok(Value::List(fields.clone()))
            } else {
                Err(RuntimeError::Custom(format!(
                    "{}: expected variant '{}', got '{}'",
                    error_message, variant_name_to_check, variant_name
                )))
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "VariantValue".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Extract data from a variant or return a default value
/// Usage: variant_or(enum_value, "VariantName", default_value) -> fields or default
fn variant_or(args: &[Value]) -> Result<Value, RuntimeError> {
    let variant_name_to_check = match &args[1] {
        Value::Text(s) => s,
        v => return Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    };

    let default_value = &args[2];

    match &args[0] {
        Value::VariantValue { variant_name, fields, .. } => {
            if variant_name == variant_name_to_check {
                // Return the fields as a list
                Ok(Value::List(fields.clone()))
            } else {
                // Return default value
                Ok(default_value.clone())
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "VariantValue".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Transform a variant if it matches, otherwise return Absent
/// Usage: refine_variant(enum_value, "VariantName", transform_fn) -> Maybe<result>
fn refine_variant(args: &[Value]) -> Result<Value, RuntimeError> {
    let variant_name_to_check = match &args[1] {
        Value::Text(s) => s,
        v => return Err(RuntimeError::TypeError {
            expected: "Text".to_string(),
            got: v.type_name().to_string(),
        }),
    };

    let transform_fn = &args[2];

    match &args[0] {
        Value::VariantValue { variant_name, fields, .. } => {
            if variant_name == variant_name_to_check {
                // Apply the transform function to the fields (as a list)
                let fields_list = Value::List(fields.clone());
                
                // Call the function with the fields
                match transform_fn {
                    Value::Chant { params: _params, body: _body, closure: _closure } => {
                        // For simplicity, we'll just return Present with the fields
                        // In a full implementation, we'd evaluate the function
                        Ok(Value::Maybe {
                            present: true,
                            value: Some(Box::new(fields_list)),
                        })
                    }
                    Value::NativeChant(native_fn) => {
                        // Call the native function
                        let result = (native_fn.func)(&[fields_list])?;
                        Ok(Value::Maybe {
                            present: true,
                            value: Some(Box::new(result)),
                        })
                    }
                    _ => Err(RuntimeError::TypeError {
                        expected: "Chant".to_string(),
                        got: transform_fn.type_name().to_string(),
                    }),
                }
            } else {
                // Variant doesn't match, return Absent
                Ok(Value::Maybe {
                    present: false,
                    value: None,
                })
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "VariantValue".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// ITERATOR FUNCTIONS - Phase 1
// ============================================================================

/// Create an iterator from a list or range
fn iter_create(args: &[Value]) -> Result<Value, RuntimeError> {
    use crate::eval::IteratorState;

    match &args[0] {
        Value::List(elements) => Ok(Value::Iterator {
            iterator_type: "List".to_string(),
            state: Box::new(IteratorState::List {
                elements: elements.clone(),
                index: 0,
            }),
        }),
        Value::Range { start, end } => {
            match (start.as_ref(), end.as_ref()) {
                (Value::Number(s), Value::Number(e)) => Ok(Value::Iterator {
                    iterator_type: "Range".to_string(),
                    state: Box::new(IteratorState::Range {
                        current: *s,
                        end: *e,
                        step: 1.0,
                    }),
                }),
                _ => Err(RuntimeError::TypeError {
                    expected: "Range with Numbers".to_string(),
                    got: "Range with non-numbers".to_string(),
                }),
            }
        }
        v => Err(RuntimeError::TypeError {
            expected: "List or Range".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get next value from iterator
fn iter_next(args: &[Value]) -> Result<Value, RuntimeError> {
    use crate::eval::IteratorState;

    match &args[0] {
        Value::Iterator { iterator_type, state } => {
            let mut state_clone = (**state).clone();

            let (maybe_value, updated_state) = match &mut state_clone {
                IteratorState::List { elements, index } => {
                    if *index < elements.len() {
                        let value = elements[*index].clone();
                        *index += 1;
                        (
                            Value::Maybe {
                                present: true,
                                value: Some(Box::new(value)),
                            },
                            state_clone,
                        )
                    } else {
                        (
                            Value::Maybe {
                                present: false,
                                value: None,
                            },
                            state_clone,
                        )
                    }
                }
                IteratorState::Range { current, end, step } => {
                    if *current < *end {
                        let value = *current;
                        *current += *step;
                        (
                            Value::Maybe {
                                present: true,
                                value: Some(Box::new(Value::Number(value))),
                            },
                            state_clone,
                        )
                    } else {
                        (
                            Value::Maybe {
                                present: false,
                                value: None,
                            },
                            state_clone,
                        )
                    }
                }
                IteratorState::Empty => (
                    Value::Maybe {
                        present: false,
                        value: None,
                    },
                    state_clone,
                ),
                _ => {
                    return Err(RuntimeError::Custom(
                        "iter_next: Advanced iterator types not yet implemented from native code".to_string()
                    ))
                }
            };

            // Return a list: [updated_iterator, maybe_value]
            let updated_iterator = Value::Iterator {
                iterator_type: iterator_type.clone(),
                state: Box::new(updated_state),
            };

            Ok(Value::List(vec![updated_iterator, maybe_value]))
        }
        v => Err(RuntimeError::TypeError {
            expected: "Iterator".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Create a mapping iterator
fn iter_map(args: &[Value]) -> Result<Value, RuntimeError> {
    use crate::eval::IteratorState;

    match (&args[0], &args[1]) {
        (Value::Iterator { .. }, func @ Value::Chant { .. }) |
        (Value::Iterator { .. }, func @ Value::NativeChant(_)) => {
            Ok(Value::Iterator {
                iterator_type: "Map".to_string(),
                state: Box::new(IteratorState::Map {
                    inner: Box::new(args[0].clone()),
                    func: Box::new(func.clone()),
                }),
            })
        }
        (Value::Iterator { .. }, v) => Err(RuntimeError::TypeError {
            expected: "Function".to_string(),
            got: v.type_name().to_string(),
        }),
        (v, _) => Err(RuntimeError::TypeError {
            expected: "Iterator".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Create a filtering iterator
fn iter_filter(args: &[Value]) -> Result<Value, RuntimeError> {
    use crate::eval::IteratorState;

    match (&args[0], &args[1]) {
        (Value::Iterator { .. }, func @ Value::Chant { .. }) |
        (Value::Iterator { .. }, func @ Value::NativeChant(_)) => {
            Ok(Value::Iterator {
                iterator_type: "Filter".to_string(),
                state: Box::new(IteratorState::Filter {
                    inner: Box::new(args[0].clone()),
                    predicate: Box::new(func.clone()),
                }),
            })
        }
        (Value::Iterator { .. }, v) => Err(RuntimeError::TypeError {
            expected: "Function".to_string(),
            got: v.type_name().to_string(),
        }),
        (v, _) => Err(RuntimeError::TypeError {
            expected: "Iterator".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Fold an iterator into a single value
fn iter_fold(_args: &[Value]) -> Result<Value, RuntimeError> {
    // Note: This function signature is (iterator, init, func)
    // The actual reduction logic needs to be implemented in the evaluator
    // because it requires calling functions dynamically
    Err(RuntimeError::Custom(
        "iter_fold: Must be implemented in Glimmer-Weave code, not as native builtin".to_string()
    ))
}

/// Collect an iterator into a list
fn iter_collect(_args: &[Value]) -> Result<Value, RuntimeError> {
    // Note: This needs to be implemented in Glimmer-Weave code
    // because it requires repeatedly calling iter_next
    Err(RuntimeError::Custom(
        "iter_collect: Must be implemented in Glimmer-Weave code, not as native builtin".to_string()
    ))
}

/// Create a take iterator
fn iter_take(args: &[Value]) -> Result<Value, RuntimeError> {
    use crate::eval::IteratorState;

    match (&args[0], &args[1]) {
        (Value::Iterator { .. }, Value::Number(n)) => {
            Ok(Value::Iterator {
                iterator_type: "Take".to_string(),
                state: Box::new(IteratorState::Take {
                    inner: Box::new(args[0].clone()),
                    remaining: *n as usize,
                }),
            })
        }
        (Value::Iterator { .. }, v) => Err(RuntimeError::TypeError {
            expected: "Number".to_string(),
            got: v.type_name().to_string(),
        }),
        (v, _) => Err(RuntimeError::TypeError {
            expected: "Iterator".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

// ============================================================================
// SMART POINTER FUNCTIONS
// ============================================================================

/// Create a new Shared<T> smart pointer
/// Usage: Shared_new(value) -> Shared<T>
fn shared_new(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Shared {
        value: Box::new(args[0].clone()),
        ref_count: 1,
    })
}

/// Get the value from a Shared<T> smart pointer
/// Usage: Shared_get(shared) -> T
fn shared_get(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Shared { value, .. } => Ok((**value).clone()),
        v => Err(RuntimeError::TypeError {
            expected: "Shared".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Clone a Shared<T> smart pointer (increments reference count)
/// Usage: Shared_clone(shared) -> Shared<T>
fn shared_clone(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Shared { value, ref_count } => Ok(Value::Shared {
            value: value.clone(),
            ref_count: ref_count + 1,
        }),
        v => Err(RuntimeError::TypeError {
            expected: "Shared".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Get the reference count of a Shared<T> smart pointer
/// Usage: Shared_count(shared) -> Number
fn shared_count(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Shared { ref_count, .. } => Ok(Value::Number(*ref_count as f64)),
        v => Err(RuntimeError::TypeError {
            expected: "Shared".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Create a new Cell<T> for interior mutability
/// Usage: Cell_new(value) -> Cell<T>
fn cell_new(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Cell {
        value: Box::new(args[0].clone()),
        borrowed: false,
        borrow_count: 0,
    })
}

/// Get the value from a Cell<T> (immutable borrow)
/// Usage: Cell_get(cell) -> T
fn cell_get(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Cell { value, borrowed, .. } => {
            if *borrowed {
                return Err(RuntimeError::Custom(
                    "Cannot get from Cell: already borrowed mutably".to_string()
                ));
            }
            Ok((**value).clone())
        }
        v => Err(RuntimeError::TypeError {
            expected: "Cell".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Set the value in a Cell<T> (mutable borrow)
/// Usage: Cell_set(cell, new_value) -> Nothing
fn cell_set(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Cell { borrowed, borrow_count, .. } => {
            if *borrowed {
                return Err(RuntimeError::Custom(
                    "Cannot set Cell: already borrowed mutably".to_string()
                ));
            }
            if *borrow_count > 0 {
                return Err(RuntimeError::Custom(
                    "Cannot set Cell: currently borrowed immutably".to_string()
                ));
            }
            // In a real implementation, we'd mutate the cell in place
            // For now, we'll return Nothing to indicate success
            // The actual mutation would need to happen via a mutable reference
            Ok(Value::Nothing)
        }
        v => Err(RuntimeError::TypeError {
            expected: "Cell".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Borrow the value immutably from a Cell<T>
/// Usage: Cell_borrow(cell) -> T
fn cell_borrow(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Cell { value, borrowed, .. } => {
            if *borrowed {
                return Err(RuntimeError::Custom(
                    "Cannot borrow: already borrowed mutably".to_string()
                ));
            }
            // In a real implementation, we'd increment borrow_count
            // For now, just return the value
            Ok((**value).clone())
        }
        v => Err(RuntimeError::TypeError {
            expected: "Cell".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Borrow the value mutably from a Cell<T>
/// Usage: Cell_borrow_mut(cell) -> T
fn cell_borrow_mut(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Cell { value, borrowed, borrow_count } => {
            if *borrowed {
                return Err(RuntimeError::Custom(
                    "Cannot borrow mutably: already borrowed mutably".to_string()
                ));
            }
            if *borrow_count > 0 {
                return Err(RuntimeError::Custom(
                    "Cannot borrow mutably: currently borrowed immutably".to_string()
                ));
            }
            // In a real implementation, we'd set borrowed = true
            // For now, just return the value
            Ok((**value).clone())
        }
        v => Err(RuntimeError::TypeError {
            expected: "Cell".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}

/// Release a borrow on a Cell<T>
/// Usage: Cell_release(cell) -> Nothing
fn cell_release(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Cell { .. } => {
            // In a real implementation, we'd decrement borrow_count or set borrowed = false
            // For now, just return Nothing
            Ok(Value::Nothing)
        }
        v => Err(RuntimeError::TypeError {
            expected: "Cell".to_string(),
            got: v.type_name().to_string(),
        }),
    }
}
