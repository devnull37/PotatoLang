use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::env::args;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
enum Token {
    Print(String),
    Var(String, String),
    Loop(Vec<Token>),
    While(String, Vec<Token>),
    Input(String, String),
    If(String, Vec<Token>),  // Added If variant
    QuitLoop,
}

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "print" => {
                let text = parts[1..].join(" ");
                tokens.push(Token::Print(text));
            }
            "new" => {
                if parts[1] == "var" {
                    let name = parts[2].to_string();
                    let value = parts[4..].join(" ");
                    tokens.push(Token::Var(name, value));
                }
            }
            "in" => {
                if parts.len() >= 4 && (parts[1] == "console" || parts[1] == "con") {
                    let var_name = parts[2].to_string();
                    let var_type = parts[3].to_string(); // Expecting type as the fourth part
                    tokens.push(Token::Input(var_name, var_type));
                } else {
                    println!("Invalid input command syntax. Use 'in console <your_variable_name> <type>'");
                }
            }
            "loop" => {
                if parts[1] == "do" {
                    i += 1;
                    let mut loop_body = Vec::new();
                    while i < lines.len() && lines[i].trim() != "quit_loop" {
                        loop_body.push(lines[i].to_string());
                        i += 1;
                    }
                    let loop_tokens = lex(&loop_body.join("\n"));
                    tokens.push(Token::Loop(loop_tokens));
                }
            }
            "while" => {
                let condition = parts[1..].join(" ");
                i += 1;
                let mut loop_body = Vec::new();
                while i < lines.len() && lines[i].trim() != "}" {
                    loop_body.push(lines[i].to_string());
                    i += 1;
                }
                let loop_tokens = lex(&loop_body.join("\n"));
                tokens.push(Token::While(condition.trim().to_string(), loop_tokens));
            }
            "if" => {
                let condition = parts[1..].join(" ");
                i += 1;
                let mut if_body = Vec::new();
                while i < lines.len() && lines[i].trim() != "}" {
                    if_body.push(lines[i].to_string());
                    i += 1;
                }
                let if_tokens = lex(&if_body.join("\n"));
                tokens.push(Token::If(condition.trim().to_string(), if_tokens));
            }
            "quit_loop" => {
                tokens.push(Token::QuitLoop);
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
                if text.starts_with("\"") && text.ends_with("\"") {
                    code.push_str(&format!("  println!({});\n", text));
                } else {
                    code.push_str(&format!("  println!(\"{{}}\", {});\n", text));
                }
            }
            Token::Var(name, value) => {
                code.push_str(&format!("  let {} = {};\n", name, value));
            }
            Token::Loop(body) => {
                code.push_str("  loop {\n");
                for body_token in body {
                    code.push_str(&transpile(&[body_token.clone()]));
                }
                code.push_str("  }\n");
            }
            Token::While(condition, body) => {
                code.push_str(&format!("  while {} {{\n", condition));
                for body_token in body {
                    code.push_str(&transpile(&[body_token.clone()]));
                }
                code.push_str("  }\n");
            }
            Token::If(condition, body) => {
                code.push_str(&format!("  if {} {{\n", condition));
                for body_token in body {
                    code.push_str(&transpile(&[body_token.clone()]));
                }
                code.push_str("  }\n");
            }
            Token::Input(var_name, var_type) => {
                code.push_str("  use std::io;\n");
                code.push_str(&format!("  let mut {} = String::new();\n", var_name));
                code.push_str("  println!(\"Enter input: \");\n");
                code.push_str(&format!("  io::stdin().read_line(&mut {}).expect(\"Failed to read line\");\n", var_name));
                if var_type == "int" {
                    code.push_str(&format!("  let {}: i32 = {}.trim().parse().expect(\"Please type a number!\");\n", var_name, var_name));
                }
            }
            Token::QuitLoop => {
                code.push_str("  break;\n");
            }
        }
    }
    code.push_str("}\n");
    code
}

fn create_new_file() {
    let mut file = File::create("main.ptl").expect("Failed to create file");
    file.write_all(br#"print "Hello, World""#).expect("Failed to write to file");
    println!("Created new Potatolang file: main.ptl");
}

fn compile_file() {
    let mut args = args().skip(1);
    let file_path = args.next().unwrap_or_else(|| String::from("main.ptl"));

    let input = match read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => panic!("Failed to read file: {}", err),
    };

    let tokens = lex(&input);
    let rust_code = transpile(&tokens);

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

const VERSION: &str = "0.1.0";

fn main() {
    println!("Potatolang Compiler v{}", VERSION);
    println!("1. New Potatolang File (Hello World)");
    println!("2. Compile Potatolang File");
    println!("3. Settings");
    println!("4. Exit");

    loop {
        println!("Enter your choice: ");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        let choice = choice.trim().parse::<u8>();

        match choice {
            Ok(1) => create_new_file(),
            Ok(2) => compile_file(),
            Ok(3) => println!("Settings not yet implemented"),
            Ok(4) => break,
            Err(_) => println!("Invalid choice. Please enter a number between 1 and 4."),
            _ => unreachable!(),
        }
    }
}
