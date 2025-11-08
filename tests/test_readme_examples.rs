// Test file to verify all README examples work correctly

use glimmer_weave::{Lexer, Parser, Evaluator, Value};

fn eval_source(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("{:?}", e))?;
    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast).map_err(|e| format!("{:?}", e))
}

#[test]
fn test_hello_world() {
    let source = r#"
        "Hello, World!"
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("Hello, World!".to_string()));
}

#[test]
fn test_factorial_function() {
    let source = r#"
        chant factorial(n) then
            should n <= 1 then
                yield 1
            otherwise
                yield n * factorial(n - 1)
            end
        end

        factorial(5)
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(120.0));
}

#[test]
fn test_data_pipeline() {
    let source = r#"
        chant double(x) then yield x * 2 end
        chant add_one(x) then yield x + 1 end

        5 | double | add_one
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(11.0));
}

#[test]
fn test_variables_immutable() {
    let source = r#"
        bind name to "Alice"
        bind age to 30
        bind pi to 3.14159

        pi
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(3.14159));
}

#[test]
fn test_variables_mutable() {
    let source = r#"
        weave counter as 0
        set counter to counter + 1
        set counter to counter + 1
        counter
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_conditionals() {
    let source = r#"
        bind age to 25
        should age >= 18 then
            "Adult"
        otherwise
            "Minor"
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("Adult".to_string()));
}

#[test]
fn test_for_each_loop() {
    let source = r#"
        bind items to [1, 2, 3, 4, 5]
        weave sum as 0

        for each item in items then
            set sum to sum + item
        end

        sum
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_while_loop() {
    let source = r#"
        weave count as 0
        whilst count less than 10 then
            set count to count + 1
        end
        count
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_break_statement() {
    let source = r#"
        weave result as []
        for each item in [1, 2, 3, 4, 5] then
            should item is 4 then
                break
            end

            set result to list_push(result, item)
        end
        result
    "#;

    let result = eval_source(source).expect("Should succeed");
    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
        },
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_continue_statement() {
    let source = r#"
        weave result as []
        for each item in [1, 2, 3, 4, 5] then
            should item is 3 then
                continue
            end

            set result to list_push(result, item)
        end
        result
    "#;

    let result = eval_source(source).expect("Should succeed");
    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(4.0));
            assert_eq!(items[3], Value::Number(5.0));
        },
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_range() {
    let source = r#"
        weave sum as 0
        for each i in range(1, 11) then
            set sum to sum + i
        end
        sum
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(55.0)); // 1+2+3+...+10 = 55
}

#[test]
fn test_function_definition() {
    let source = r#"
        chant greet(name) then
            yield "Hello, " + name + "!"
        end

        bind message to greet("Alice")
        message
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("Hello, Alice!".to_string()));
}

#[test]
fn test_variadic_function() {
    let source = r#"
        chant sum(...numbers) then
            weave total as 0
            for each n in numbers then
                set total to total + n
            end
            yield total
        end

        bind result to sum(1, 2, 3, 4, 5)
        result
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_closure() {
    let source = r#"
        bind x to 10

        chant add_x(y) then
            yield x + y
        end

        add_x(5)
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_pattern_matching_literals() {
    let source = r#"
        bind number to 42
        match number with
            when 1 then "one"
            when 2 then "two"
            when 42 then "the answer"
            when _ then "something else"
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("the answer".to_string()));
}

#[test]
fn test_pattern_matching_enums() {
    let source = r#"
        bind result to Present(42)

        match result with
            when Present(value) then
                "Found: " + to_text(value)
            when Absent then
                "Not found"
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("Found: 42".to_string()));
}

#[test]
fn test_error_handling_outcome() {
    let source = r#"
        chant divide(a, b) then
            should b is 0 then
                yield Mishap("Division by zero")
            otherwise
                yield Triumph(a / b)
            end
        end

        bind result to divide(10, 2)
        match result with
            when Triumph(value) then
                "Result: " + to_text(value)
            when Mishap(error) then
                "Error: " + error
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("Result: 5".to_string()));
}

#[test]
fn test_error_propagation_operator() {
    let source = r#"
        chant divide(a, b) then
            should b is 0 then
                yield Mishap("Division by zero")
            otherwise
                yield Triumph(a / b)
            end
        end

        chant safe_divide(a, b) then
            bind result to divide(a, b)?
            yield Triumph(result * 2)
        end

        bind outcome to safe_divide(10, 2)
        match outcome with
            when Triumph(value) then value
            when Mishap(error) then 0
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(10.0)); // (10/2) * 2 = 10
}

#[test]
fn test_struct_definition() {
    let source = r#"
        form Person with
            name as Text
            age as Number
            city as Text
        end

        bind alice to { name: "Alice", age: 30, city: "Seattle" }

        bind alice_name to alice["name"]
        alice_name
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("Alice".to_string()));
}

#[test]
fn test_nested_structs() {
    let source = r#"
        form Point with
            x as Number
            y as Number
        end

        form Rectangle with
            top_left as Point
            bottom_right as Point
        end

        bind rect to {
            top_left: { x: 0, y: 10 },
            bottom_right: { x: 5, y: 0 }
        }

        bind width to rect["bottom_right"]["x"] - rect["top_left"]["x"]
        width
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_iterator_basic() {
    let source = r#"
        # Simple iterator test with iter_next on base iterator
        bind numbers to [1, 2, 3, 4, 5]
        bind it to iter(numbers)

        # Get next value from basic iterator
        bind pair to iter_next(it)
        bind next_iter to list_first(pair)
        bind next_value to list_last(pair)

        match next_value with
            when Present(value) then value
            when Absent then 0
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(1.0));  // First element
}

#[test]
fn test_fizzbuzz() {
    let source = r#"
        chant fizzbuzz(n) then
            should n % 15 is 0 then
                yield "FizzBuzz"
            otherwise
                should n % 3 is 0 then
                    yield "Fizz"
                otherwise
                    should n % 5 is 0 then
                        yield "Buzz"
                    otherwise
                        yield to_text(n)
                    end
                end
            end
        end

        weave results as []
        for each i in range(1, 16) then
            set results to list_push(results, fizzbuzz(i))
        end
        results
    "#;

    let result = eval_source(source).expect("Should succeed");
    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 15);
            assert_eq!(items[0], Value::Text("1".to_string()));
            assert_eq!(items[1], Value::Text("2".to_string()));
            assert_eq!(items[2], Value::Text("Fizz".to_string()));
            assert_eq!(items[4], Value::Text("Buzz".to_string()));
            assert_eq!(items[14], Value::Text("FizzBuzz".to_string()));
        },
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_list_processing_with_loops() {
    let source = r#"
        bind numbers to [-2, -1, 0, 1, 2, 3, 4, 5]

        weave result as []
        for each n in numbers then
            should n greater than 0 then
                set result to list_push(result, n * 2)
            end
        end

        result
    "#;

    let result = eval_source(source).expect("Should succeed");
    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 5);
            assert_eq!(items[0], Value::Number(2.0));
            assert_eq!(items[1], Value::Number(4.0));
            assert_eq!(items[2], Value::Number(6.0));
            assert_eq!(items[3], Value::Number(8.0));
            assert_eq!(items[4], Value::Number(10.0));
        },
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_builtin_functions_list() {
    let source = r#"
        bind nums to [1, 2, 3]
        bind count to list_length(nums)
        bind pushed to list_push(nums, 4)
        bind first_val to list_first(pushed)
        bind last_val to list_last(pushed)

        list_sum([count, first_val, last_val])
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(8.0)); // 3 + 1 + 4 = 8
}

#[test]
fn test_builtin_functions_string() {
    let source = r#"
        bind text to "hello"
        bind text_upper to upper(text)
        bind len to length(text_upper)
        bind has_ello to contains(text_upper, "ELLO")

        should has_ello then
            to_text(len)
        otherwise
            "not found"
        end
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Text("5".to_string()));
}

#[test]
fn test_builtin_functions_math() {
    let source = r#"
        bind x to floor(3.7)
        bind y to ceil(3.2)
        bind z to round(3.5)
        bind w to abs(-5.3)

        x + y + z + w
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(16.3)); // 3 + 4 + 4 + 5.3 = 16.3
}

#[test]
fn test_pipeline_operator() {
    let source = r#"
        chant double(x) then yield x * 2 end
        chant add_one(x) then yield x + 1 end
        chant square(x) then yield x * x end

        5 | double | add_one | square
    "#;

    let result = eval_source(source).expect("Should succeed");
    assert_eq!(result, Value::Number(121.0)); // ((5 * 2) + 1)² = 121
}
