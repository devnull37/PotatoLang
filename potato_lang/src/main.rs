use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::env::args;
use std::fs::read_to_string;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone)]
enum Token {
    Print(String),
    Var(String, String),
    Func(String, Vec<String>, Vec<Token>),
    Call(String),
    Loop(Vec<Token>),
    While(String, Vec<Token>),
    Input(String, String),
    If(String, Vec<Token>),
    Sleep(String),
    QuitLoop,
}

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() { i += 1; continue; }

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "print" => {
                if parts.len() > 2 && parts[1] == "to" && parts[2] == "terminal" {
                    let text = parts[3..].join(" ").replace("\\n", "\n");
                    tokens.push(Token::Print(text));
                } else {
                    println!("Invalid print syntax. Use 'print to terminal <text>'");
                }
            }
            "new" => {
                if parts.len() > 4 && parts[1] == "var" && parts[3] == "=" {
                    let name = parts[2].to_string();
                    let value = parts[4..].join(" "); // Supports math like "y + z"
                    tokens.push(Token::Var(name, value));
                } else {
                    println!("Invalid var syntax. Use 'new var [name] = [value]'");
                }
            }
            "func" => {
                let name = parts[1].to_string();
                let params: Vec<String> = parts[2].split(',').map(|s| s.trim().to_string()).collect();
                i += 1;
                let mut func_body = Vec::new();
                while i < lines.len() && lines[i].trim() != "endfunc" {
                    func_body.push(lines[i].to_string());
                    i += 1;
                }
                let func_tokens = lex(&func_body.join("\n"));
                tokens.push(Token::Func(name, params, func_tokens));
            }
            "call" => {
                let func_name = parts[1].to_string();
                tokens.push(Token::Call(func_name));
            }
            "in" => {
                if parts.len() >= 4 && (parts[1] == "con" || parts[1] == "console") {
                    let var_name = parts[2].to_string();
                    let var_type = parts[3].to_string();
                    tokens.push(Token::Input(var_name, var_type));
                } else {
                    println!("Invalid input syntax. Use 'in con [var] [type]' or 'in console [var] [type]'");
                }
            }
            "loop" => {
                if parts.get(1) == Some(&"do") {
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
            "sleep" => {
                let duration = parts[1].to_string();
                tokens.push(Token::Sleep(duration));
            }
            "quit_loop" => tokens.push(Token::QuitLoop),
            _ => println!("Warning: Unknown command '{}'", parts[0]),
        }
        i += 1;
    }
    tokens
}

fn transpile(tokens: &[Token]) -> String {
    let mut code = String::new();
    let mut functions = String::new();
    code.push_str("use std::io;\n");
    code.push_str("use std::thread::sleep;\n");
    code.push_str("use std::time::Duration;\n");
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
                code.push_str(&format!("  let mut {} = {};\n", name, value));
            }
            Token::Func(name, params, body) => {
                functions.push_str(&format!("fn {}({}) {{\n", 
                    name,
                    params.iter().map(|p| format!("{}: i32", p)).collect::<Vec<String>>().join(", ")
                ));
                for body_token in body {
                    functions.push_str(&transpile(&[body_token.clone()]));
                }
                functions.push_str("}\n");
            }
            Token::Call(func_name) => {
                code.push_str(&format!("  {}();\n", func_name));
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
                code.push_str(&format!("  let mut {} = String::new();\n", var_name));
                code.push_str("  println!(\"Enter input: \");\n");
                code.push_str(&format!("  io::stdin().read_line(&mut {}).expect(\"Failed to read line\");\n", var_name));
                match var_type.as_str() {
                    "int" => code.push_str(&format!("  let {}: i32 = {}.trim().parse().expect(\"Please type a number!\");\n", var_name, var_name)),
                    "str" => code.push_str(&format!("  let {} = {}.trim().to_string();\n", var_name, var_name)),
                    _ => code.push_str(&format!("  let {} = {};\n", var_name, var_name)),
                }
            }
            Token::Sleep(duration) => {
                code.push_str(&format!("  sleep(Duration::from_millis({}));\n", duration));
            }
            Token::QuitLoop => code.push_str("  break;\n"),
        }
    }
    code.push_str("}\n");
    code.push_str(&functions);
    code
}

fn create_new_file() {
    let mut file = File::create("main.ptl").expect("Failed to create file");
    file.write_all(br#"print to terminal "Hello, World"
sleep 1000
print to terminal "After 1 second""#).expect("Failed to write to file");
    println!("Created new Potatolang file: main.ptl");
}

fn compile_file() {
    let mut args = args().skip(1);
    let file_path = args.next().unwrap_or_else(|| String::from("main.ptl"));

    let input = match read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => panic!("Failed to read file '{}': {}", file_path, err),
    };

    let tokens = lex(&input);
    let rust_code = transpile(&tokens);

    let mut rust_file = File::create("output.rs").expect("Cannot create file");
    rust_file.write_all(rust_code.as_bytes()).expect("Cannot write to file");
    println!("Rust file generated successfully.");
    println!("Generated Rust code:\n{}", rust_code);

    println!("Compiling Rust code...");
    let output = Command::new("rustc")
        .arg("output.rs")
        .output()
        .expect("Failed to compile Rust code");
    
    if !output.status.success() {
        println!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }
    println!("Compilation successful!");

    println!("Running Rust code...");
    let output = Command::new("./output")
        .output()
        .expect("Failed to run Rust code");
    println!("Execution output: {}", String::from_utf8_lossy(&output.stdout));
}

const VERSION: &str = "0.2.0";

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
        match choice.trim().parse::<u8>() {
            Ok(1) => create_new_file(),
            Ok(2) => compile_file(),
            Ok(3) => println!("Settings not yet implemented"),
            Ok(4) => break,
            Ok(_) => println!("Invalid choice. Please enter a number between 1 and 4."),
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    }
}