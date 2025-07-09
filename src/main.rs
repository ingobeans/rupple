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
    // yes this is not a proper way to do this but it works most of the time, okay!?
    // basically we just check if the input is parseable as a TokenStream
    // this happens to fail on, say, an unterminated closure.
    // although it does also have a few edge cases where it also fails,
    // such as randomly typing a closing bracket, or typing a char literal, but with multiple chars, like: `let a = 'abc';`
    // but for now i'll leave this as is, because it does work, and false positives are rare, and always the user's fault in some way
    input.parse::<proc_macro2::TokenStream>().is_err()
}

/// Function that checks for the specific case where input ends with a let declaration without a terminating semicolon
fn requires_extra_semicolon(mut tokens: Vec<TokenTree>) -> bool {
    // if theres at least 3 tokens
    if tokens.len() >= 3 {
        // and the ultimate token is a literal
        if let TokenTree::Literal(_) = tokens.pop().unwrap() {
            // and the penultimate token is a punctuation character
            if let TokenTree::Punct(char) = tokens.pop().unwrap() {
                // and the penultimate token is an equals sign
                if char.as_char() == '=' {
                    // and the antepenultimate token is an identifier
                    if let TokenTree::Ident(_) = tokens.pop().unwrap() {
                        // and the preantepenultimate token is an identifier
                        if let TokenTree::Ident(sym) = tokens.pop().unwrap() {
                            // and the preantepenultimate token's text == "let"
                            if sym.to_string().trim() == "let" {
                                // then that means we are looking at a let declaration without a terminating semicolon
                                // OH NO!!
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    // or if any of those conditions fail, we're all good
    false
}

/// Formats input code with boilerplate base contents
fn format(mut input: String) -> String {
    let tokens = input.parse::<TokenStream>().unwrap().into_iter().collect();
    if requires_extra_semicolon(tokens) {
        input += ";"
    }

    if !input.trim().ends_with(";") {
        input = format!(
            "let output = {{\n{}\n}}; FalliblePrinter(output).print();",
            input
        )
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
