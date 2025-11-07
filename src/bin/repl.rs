/// Glimmer-Weave REPL (Read-Eval-Print Loop)
/// Interactive shell for rapid prototyping and testing code snippets

use glimmer_weave::{Evaluator, Lexer, Parser};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

const WELCOME_MESSAGE: &str = r#"
╔══════════════════════════════════════════════════════════════╗
║             Glimmer-Weave REPL v0.1.0                        ║
║     "The code should read like poetry, not algebra."         ║
╚══════════════════════════════════════════════════════════════╝

Type :help for help, :quit to exit

"#;

const HELP_MESSAGE: &str = r#"
Glimmer-Weave REPL Commands:

  :help         Show this help message
  :quit, :exit  Exit the REPL
  :clear        Clear the screen
  :env          Show all defined variables
  :reset        Reset the environment (clear all variables)

Examples:

  bind x to 42
  x + 10

  chant factorial(n) then
      should n <= 1 then
          yield 1
      otherwise
          yield n * factorial(n - 1)
      end
  end

  factorial(5)

  5 | chant(x) then yield x * 2 end | chant(x) then yield x + 1 end

Type any Glimmer-Weave expression and press Enter to evaluate it.
Use Ctrl+C to cancel the current input, Ctrl+D to exit.
"#;

/// Format a value for REPL display (more concise than Debug)
fn format_value(value: &glimmer_weave::eval::Value) -> String {
    use glimmer_weave::eval::Value;

    match value {
        Value::Number(n) => format!("{}", n),
        Value::Text(s) => format!("\"{}\"", s),
        Value::Truth(b) => format!("{}", b),
        Value::Nothing => "nothing".to_string(),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(|v| format_value(v)).collect();
            format!("[{}]", formatted.join(", "))
        }
        Value::Map(map) => {
            let formatted: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{{}}}", formatted.join(", "))
        }
        Value::Chant { .. } => "<function>".to_string(),
        Value::NativeChant(nf) => format!("<native function: {}>", nf.name),
        Value::StructInstance { struct_name, fields } => {
            let formatted: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{} {{ {} }}", struct_name, formatted.join(", "))
        }
        Value::Maybe { present, value } => {
            if *present {
                if let Some(v) = value {
                    format!("Present({})", format_value(v))
                } else {
                    "Present".to_string()
                }
            } else {
                "Absent".to_string()
            }
        }
        Value::Outcome { success, value } => {
            if *success {
                format!("Triumph({})", format_value(value))
            } else {
                format!("Mishap({})", format_value(value))
            }
        }
        Value::VariantValue { enum_name: _, variant_name, fields, type_args: _ } => {
            if fields.is_empty() {
                variant_name.clone()
            } else {
                let formatted: Vec<String> = fields.iter().map(|v| format_value(v)).collect();
                format!("{}({})", variant_name, formatted.join(", "))
            }
        }
        Value::VariantConstructor { enum_name, variant_name, .. } => {
            format!("<variant constructor: {}::{})", enum_name, variant_name)
        }
        Value::Iterator { .. } => "<iterator>".to_string(),
        Value::Capability { resource, .. } => format!("<capability: {}>", resource),
        Value::Range { start, end } => format!("range({}, {})", format_value(start), format_value(end)),
        Value::StructDef { name, .. } => format!("<struct definition: {}>", name),
        Value::VariantDef { name, .. } => format!("<enum definition: {}>", name),
    }
}

fn main() -> Result<()> {
    // Print welcome message
    println!("{}", WELCOME_MESSAGE);

    // Create line editor with history
    let mut rl = DefaultEditor::new()?;

    // Load history from file
    let history_file = dirs::data_local_dir()
        .map(|mut p| {
            p.push("glimmer-weave");
            std::fs::create_dir_all(&p).ok();
            p.push("history.txt");
            p
        });

    if let Some(ref history_path) = history_file {
        let _ = rl.load_history(history_path);
    }

    // Create evaluator (maintains state across REPL sessions)
    let mut evaluator = Evaluator::new();

    // Buffer for multi-line input
    let mut input_buffer = String::new();
    let mut line_number = 1;

    loop {
        // Determine prompt based on whether we're in multi-line mode
        let prompt = if input_buffer.is_empty() {
            format!("glimmer[{}]> ", line_number)
        } else {
            format!("       ...> ")
        };

        // Read line
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                // Handle empty lines
                if line.trim().is_empty() {
                    if !input_buffer.is_empty() {
                        // Empty line in multi-line mode continues
                        input_buffer.push('\n');
                    }
                    continue;
                }

                // Handle special commands
                if line.trim().starts_with(':') {
                    match line.trim() {
                        ":quit" | ":exit" => {
                            println!("Goodbye!");
                            break;
                        }
                        ":help" => {
                            println!("{}", HELP_MESSAGE);
                            continue;
                        }
                        ":clear" => {
                            print!("\x1B[2J\x1B[1;1H");
                            continue;
                        }
                        ":env" => {
                            println!("Environment variables:");
                            // We'd need to add a method to Evaluator to list variables
                            // For now, just print a message
                            println!("  (environment inspection not yet implemented)");
                            continue;
                        }
                        ":reset" => {
                            evaluator = Evaluator::new();
                            println!("Environment reset.");
                            continue;
                        }
                        cmd => {
                            println!("Unknown command: {}", cmd);
                            println!("Type :help for available commands.");
                            continue;
                        }
                    }
                }

                // Add line to history
                rl.add_history_entry(line.as_str())?;

                // Append to buffer
                if !input_buffer.is_empty() {
                    input_buffer.push('\n');
                }
                input_buffer.push_str(&line);

                // Try to parse and evaluate
                match try_eval(&mut evaluator, &input_buffer) {
                    Ok(result) => {
                        // Successfully evaluated
                        println!("{}", format_value(&result));
                        input_buffer.clear();
                        line_number += 1;
                    }
                    Err(EvalError::Incomplete) => {
                        // Need more input (multi-line)
                        continue;
                    }
                    Err(EvalError::Parse(msg)) => {
                        println!("Parse error: {}", msg);
                        input_buffer.clear();
                    }
                    Err(EvalError::Runtime(msg)) => {
                        println!("Runtime error: {}", msg);
                        input_buffer.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - clear current input
                if !input_buffer.is_empty() {
                    println!("^C");
                    input_buffer.clear();
                } else {
                    println!("(To exit, press Ctrl+D or type :quit)");
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - exit
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(ref history_path) = history_file {
        let _ = rl.save_history(history_path);
    }

    Ok(())
}

enum EvalError {
    Incomplete,
    Parse(String),
    Runtime(String),
}

fn try_eval(evaluator: &mut Evaluator, source: &str) -> std::result::Result<glimmer_weave::eval::Value, EvalError> {
    // Tokenize
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Check for common incomplete patterns
    if is_incomplete(source) {
        return Err(EvalError::Incomplete);
    }

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            // Check if parse error might indicate incomplete input
            let err_msg = format!("{:?}", e);
            if err_msg.contains("Unexpected end of input")
                || err_msg.contains("Expected")
                || source.trim().ends_with("then")
                || source.trim().ends_with("otherwise") {
                return Err(EvalError::Incomplete);
            }
            return Err(EvalError::Parse(err_msg));
        }
    };

    // Evaluate
    match evaluator.eval(&ast) {
        Ok(value) => Ok(value),
        Err(e) => Err(EvalError::Runtime(format!("{:?}", e))),
    }
}

/// Check if the input looks incomplete (heuristic)
fn is_incomplete(source: &str) -> bool {
    let trimmed = source.trim();

    // Count keywords that require 'end'
    let mut depth = 0;
    let lines: Vec<&str> = source.lines().collect();

    for line in lines {
        let line = line.trim();

        // Keywords that open a block
        if line.starts_with("chant ")
            || line.starts_with("should ")
            || line.starts_with("whilst ")
            || line.starts_with("for each ")
            || line.starts_with("match ")
            || line.starts_with("attempt ")
            || line.starts_with("form ") {
            depth += 1;
        }

        // Keywords that close a block
        if line == "end" || line.starts_with("end ") {
            depth -= 1;
        }
    }

    // If depth > 0, we have unclosed blocks
    if depth > 0 {
        return true;
    }

    // Check for lines ending with keywords that expect continuation
    if trimmed.ends_with(" then")
        || trimmed.ends_with(" otherwise")
        || trimmed.ends_with(" with")
        || trimmed.ends_with(" when") {
        return true;
    }

    false
}
