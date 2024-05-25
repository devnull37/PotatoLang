use std::fs::File;
use std::io::{self, Write};
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
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        println!("Line: {}", line); // Debug print line
        let parts: Vec<&str> = line.split_whitespace().collect();
        println!("Parts: {:?}", parts); // Debug print parts
        match parts[0] {
            "print" => {
                if parts.len() > 1 {
                    // Handle strings with multiple words (similar to before)
                    let mut text = String::new();
                    for word in parts.iter().skip(1) {
                        text.push_str(word);
                        text.push_str(" ");
                    }
                    text.pop(); // Remove trailing space
                    tokens.push(Token::Print(text.trim().to_string()));
                } else {
                    // Treat as variable name
                    tokens.push(Token::Print(parts[1].to_string()));
                }
            }
            "new" => {
                if parts[1] == "var" {
                    // Extract string value without extra space and equal sign
                    let mut value = String::new();
                    for word in parts.iter().skip(3) {
                        value.push_str(word);
                        value.push_str(" ");
                    }
                    value.pop(); // Remove trailing space
                    tokens.push(Token::Var(parts[2].to_string(), value.trim().to_string()));
                }
            }
            _ => {}
        }
        i += 1;
    }
    tokens
}



fn transpile(tokens: &[Token]) -> String {
    let mut code = String::new();
    code.push_str("fn main() {\n");
    for token in tokens {
        match token {
            Token::Print(text) => {
                println!("Processing Token::Print: {}", text); // Debug print
                if text.starts_with("\"") && text.ends_with("\"") {
                    // If it's a string literal, print it as is
                    code.push_str(&format!("  println!({});\n", text));
                } else {
                    // If it's a variable, print its value
                    code.push_str(&format!("  println!(\"{{}}\", {});\n", text));
                }
            }
            Token::Var(name, value) => {
                println!("Processing Token::Var: name = {}, value = {}", name, value); // Debug print
                code.push_str(&format!("  let {}  {};\n", name, value));
            }
            Token::Loop(actions) => {
                code.push_str("  loop {\n");
                for action in actions {
                    code.push_str(&format!("    {}\n", action));
                }
                code.push_str("    break;\n  }\n");
            }
            Token::While(condition) => {
                code.push_str(&format!("  while {} {{}}\n", condition));
            }
            Token::Math(op, result_var, left_var, right_var) => {
                code.push_str(&format!("  let {} = {} {} {};\n", result_var, left_var, op, right_var));
            }
            _ => println!("Encountered unexpected token type"), // Debug print
        }
    }
    code.push_str("}\n");
    code
}




fn main() {
    println!("Enter your PotatoLang code (end with an empty line):");
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
