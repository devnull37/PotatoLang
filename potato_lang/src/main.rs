use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::env::args;
use std::fs::read_to_string;


#[derive(Debug)]
enum Token {
    Print(String),
    Var(String, String),
    While(String),
    Input(String),
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

            "in" => {
                if parts.len() >= 3 && (parts[1] == "console" || parts[1] == "con") {
                    tokens.push(Token::Input(parts[2].to_string()));
                } else {
                    println!("Invalid input command syntax. Use 'in console <your_variable_name>'");
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
            // Token::Loop(actions) => {
            //     println!("Processing Token::Loop: actions = {:?}", actions); // Debug print
            //     code.push_str("  loop {\n"); // Loop structure with indentation
            //     for action in actions {
            //       code.push_str(&format!("    {}\n", action));
            //     }
            // }
            // code.push_str("  }\n");
            
            Token::While(condition) => {
                println!("While condition: {}", condition); // Debug print
                code.push_str(&format!("  while {} {{}}\n", condition));
              }
            Token::Input(var_name) => {
                code.push_str(&format!("use std::io;\n"));
                code.push_str(&format!("let mut {} = String::new();\n", var_name));
                code.push_str("println!(\"{symbol} \");");
                code.push_str(&format!("io::stdin().read_line(&mut {});\n", var_name));
            }
            
            _ => println!("Encountered unexpected token type"), // Debug print
        }
    }
    code.push_str("}\n");
    code
}


fn create_new_file() {
    let mut file = File::create("main.ptl").expect("Failed to create file");
    file.write_all(b"print \"Hello, Potato!\"\n").expect("Failed to write to file");
    println!("Created new Potatolang file: main.ptl");
}

fn compile_file() {
    let mut args = args().skip(1);
    let file_path = args.next().unwrap_or_else(|| String::from("main.ptl"));

    // Read file contents
    let input = match read_to_string(file_path) {
    Ok(content) => content,
    Err(err) => panic!("Failed to read file: {}", err),
};

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

const VERSION: &str = "0.1.0"; // Update with your actual version

fn main() {
    println!("Potatolang Compiler v{}", VERSION);
    println!("1. New Potatolang File (Hello World)");
    println!("2. Compile Potatolang File");
    println!("3. Settings"); // Placeholder for future settings
    println!("4. Exit");

    loop {
        println!("Enter your choice: ");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        let choice = choice.trim().parse::<u8>();

        match choice {
            Ok(1) => create_new_file(),
            Ok(2) => compile_file(),
            Ok(3) => println!("Settings not yet implemented"), // Placeholder
            Ok(4) => break,
            Err(_) => println!("Invalid choice. Please enter a number between 1 and 4."),
            _ => unreachable!(), // Should not happen
        }
    }
}