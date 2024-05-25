use std::fs::File;
use std::io::{self, Read, Write};
use std::process::Command;

#[derive(Debug)]
enum Token {
    Print(String),
    Var(String, String),
    Loop(Vec<String>),
    While(String),
    Math(String, String, String, String),
}

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    for line in lines {
        println!("Line: {}", line); // Print line for debugging
        let parts: Vec<&str> = line.split_whitespace().collect();
        println!("Parts: {:?}", parts); // Print parts for debugging
        match parts[0] {
            "print" => {
                if line.contains("\"") {
                    tokens.push(Token::Print(line[7..line.len()-2].trim().to_string())); // Extract text inside quotes
                } else {
                    tokens.push(Token::Print(parts[1..].join(" "))); // Treat as variable name
                }
            }
            
            "new" => {
                if parts[1] == "var" {
                    tokens.push(Token::Var(parts[2].to_string(), parts[4].to_string()));
                } else {
                    let mut loop_actions = Vec::new();
                    let mut i = 3;
                    while parts[i] != "quit_loop" {
                        loop_actions.push(parts[i].to_string());
                        i += 1;
                    }
                    tokens.push(Token::Loop(loop_actions));
                }
            }
            "while" => {
                let condition = format!("{} {} {}", parts[1], parts[2], parts[4]);
                tokens.push(Token::While(condition));
            }
            "+" | "-" | "*" | "/" | "%" => {
                let operation = parts[0].to_string();
                let result_var = parts[2].to_string();
                let left_var = parts[4].to_string();
                let right_var = parts[6].to_string();
                tokens.push(Token::Math(operation, result_var, left_var, right_var));
            }
            _ => {}
        }
    }
    tokens
}

fn transpile(tokens: &[Token]) -> String {
    let mut code = String::new();
    code.push_str("fn main() {\n");
    for token in tokens {
        match token {
            Token::Print(text) => {
                if text.starts_with("\"") && text.ends_with("\"") {
                    code.push_str(&format!("    println!({});\n", text));
                } else {
                    code.push_str(&format!("    println!(\"{{}}\", {});\n", text)); // Print variable without quotes and curly braces
                }
            }
            Token::Var(name, value) => {
                code.push_str(&format!("    let {} = {};\n", name, value));
            }
            Token::Loop(actions) => {
                code.push_str("    loop {\n");
                for action in actions {
                    code.push_str(&format!("        {}\n", action));
                }
                code.push_str("        break;\n    }\n");
            }

            Token::Print(text) => {
                code.push_str(&format!("    println!(\"{}\");\n", text));
            }
        
            Token::While(condition) => {
                code.push_str(&format!("    while {} {{}}\n", condition));
            }
            Token::Math(op, result_var, left_var, right_var) => {
                code.push_str(&format!("    let {} = {} {} {};\n", result_var, left_var, op, right_var));
            }
            _ => {} // Add a default case to catch any unexpected tokens
        }
    }
    code.push_str("}\n");
    code
}

fn main() {
    println!("Enter your Potatolang code (end with an empty line):");
    let mut input = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).expect("Failed to read line");
        if line.trim().is_empty() {
            break;
        }
        input.push_str(&line);
    }

    let tokens = lex(&input);
    println!("Tokens: {:?}", tokens);

    let rust_code = transpile(&tokens);
    println!("Rust code: {}", rust_code);

    let mut rust_file = File::create("output.rs").expect("Cannot create file");
    rust_file.write_all(rust_code.as_bytes()).expect("Cannot write to file");
    println!("Rust file generated successfully.");
    println!("Generated Rust code:");
    println!("{}", rust_code);

    println!("Compiling Rust code...");
    let output = Command::new("rustc")
        .arg("output.rs")
        .output()
        .expect("Failed to compile Rust code");
    println!("Compilation output: {}", String::from_utf8_lossy(&output.stderr));

    println!("Running Rust code...");
    let output = Command::new("./output")
        .output()
        .expect("Failed to run Rust code");
    println!("Execution output: {}", String::from_utf8_lossy(&output.stdout));
}
