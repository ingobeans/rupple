use std::{
    io::{stdin, stdout, Write},
    path::PathBuf,
    process::Command,
};

use crossterm::{
    execute, queue,
    style::{ResetColor, SetForegroundColor},
};
use proc_macro2::{TokenStream, TokenTree};
use rand::{rng, Rng};

const BASE_CONTENTS: &str = include_str!("base.rs");

const HELP: &str = "/help - prints help
/clear - clears repl
/exit - quits repl
/debug - prints stored repl data";

/// Whether a user's input is incomplete or not.
/// For instance, true if the user opens a closure that isn't closed,
/// or declares a string without terminating quote.
fn is_input_incomplete(input: &str) -> bool {
    // Parsing only fails if something was unterminated
    !input.parse::<proc_macro2::TokenStream>().is_ok()
}

/// Returns the leading whitespace of a str
fn get_leading_whitespace(text: &str) -> String {
    let mut buf = String::new();
    for char in text.chars() {
        if char.is_whitespace() {
            buf.push(char);
        } else {
            break;
        }
    }
    buf
}

/// Function that checks for the specific case where input ends with a let declaration without a semicolon
fn requires_extra_semicolon(mut tokens: Vec<TokenTree>) -> bool {
    if tokens.len() >= 3 {
        if let TokenTree::Literal(_) = tokens.pop().unwrap() {
            if let TokenTree::Punct(char) = tokens.pop().unwrap() {
                if char.as_char() == '=' {
                    if let TokenTree::Ident(_) = tokens.pop().unwrap() {
                        if let TokenTree::Ident(sym) = tokens.pop().unwrap() {
                            if sym.to_string().trim() == "let" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Formats input code with boilerplate base contents
fn format(mut input: String) -> String {
    let tokens = input.parse::<TokenStream>().unwrap().into_iter().collect();
    if requires_extra_semicolon(tokens) {
        input += ";"
    }

    if !input.trim().ends_with(";") {
        let mut lines: Vec<String> = input.split(";").map(|f| f.to_string()).collect();
        let last = lines.last_mut().unwrap();
        *last = format!(
            "{}FalliblePrinter({}).print();",
            get_leading_whitespace(last),
            last.trim()
        );
        input = lines.join(";");
    } else {
    }

    BASE_CONTENTS.replace("// user input", &input)
}

/// Returns success
fn run(input: String, code_path: &PathBuf, exe_path: &PathBuf, modified: bool) -> bool {
    // write file
    let formatted_file_contents = format(input);
    std::fs::write(code_path, formatted_file_contents).unwrap();

    // compile new code (only if code has been modified, or no exe exists)
    if modified || !std::fs::exists(exe_path).unwrap_or(false) {
        // compile
        let compile_process = Command::new("rustc")
            .arg(code_path)
            .arg("-o")
            .arg(exe_path)
            .arg("--color=always")
            .output()
            .unwrap();

        if !compile_process.status.success() {
            stdout().lock().write_all(&compile_process.stderr).unwrap();
            return false;
        }
    }

    // run
    Command::new(exe_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
}

/// Generate unique rupple id for temp dir,
/// formatted like "rupple-XXXXX" where "XXXXX" are random characters from the alphabet
fn generate_rupple_id() -> String {
    let mut id = String::from("rupple-");
    let mut rng = rng();

    for _ in 0..5 {
        id += &(rng.random_range(97_u8..122_u8) as char).to_string()
    }
    id
}

fn main() {
    let temp_dir = std::env::temp_dir().join(generate_rupple_id());
    std::fs::create_dir(&temp_dir).expect("couldn't create temp dir :<");

    let code_path = temp_dir.join("main.rs");
    // we use .exe for all OSes, since for windows it is required, and for linux file extension doesn't matter
    // so its fine
    let exe_path = temp_dir.join("main.exe");

    let mut stdout = stdout();

    queue!(stdout, SetForegroundColor(crossterm::style::Color::Green)).unwrap();
    println!("[rupple {}]", env!("CARGO_PKG_VERSION"));

    queue!(
        stdout,
        SetForegroundColor(crossterm::style::Color::DarkGrey)
    )
    .unwrap();
    println!("do /help for list of commands, or just start typing code!");

    // make ctrl+c clean up temp dir
    ctrlc::set_handler(move || {
        std::fs::remove_dir_all(&temp_dir).unwrap();
        std::process::exit(0);
    })
    .unwrap();

    let mut current_file_contents = String::new();
    let mut buf = String::new();
    queue!(stdout, SetForegroundColor(crossterm::style::Color::Green)).unwrap();
    print!("> ");
    loop {
        let mut modified_file_contents = current_file_contents.clone();
        execute!(stdout, ResetColor).unwrap();

        stdin().read_line(&mut buf).unwrap();
        let line_input = buf.trim();

        if is_input_incomplete(&buf) {
            print!("  ");
            continue;
        }

        if line_input.starts_with("/") && !line_input.starts_with("//") {
            match line_input {
                "/help" => {
                    println!("{HELP}")
                }
                "/clear" | "/reset" => {
                    current_file_contents = String::new();
                    if std::fs::exists(&exe_path).unwrap_or(false) {
                        std::fs::remove_file(&exe_path).unwrap();
                    }
                }
                "/exit" | "/quit" => {
                    return;
                }
                "/debug" => {
                    println!("{}", current_file_contents);
                }
                _ => {
                    println!("unknown command, do /help for list")
                }
            }
        } else {
            if !modified_file_contents.is_empty() && !modified_file_contents.trim().ends_with(";") {
                modified_file_contents += ";\n";
            }
            modified_file_contents += line_input;
            if line_input.ends_with(";") {
                modified_file_contents += "\n";
            }

            let success = run(
                modified_file_contents.clone(),
                &code_path,
                &exe_path,
                !line_input.is_empty(),
            );
            queue!(stdout, ResetColor).unwrap();

            if success {
                // only save changes if it compiled successfully
                current_file_contents = modified_file_contents;
            }
        }

        buf = String::new();
        queue!(stdout, SetForegroundColor(crossterm::style::Color::Green)).unwrap();
        print!("> ");
    }
}
