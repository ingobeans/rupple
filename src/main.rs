use std::{
    io::{stdin, stdout, Write},
    path::PathBuf,
    process::Command,
};

use crossterm::{
    execute, queue,
    style::{ResetColor, SetForegroundColor},
};

const BASE_CONTENTS: &str = "
use std::fmt::Debug;
fn rupple() -> Option<Box<dyn Debug>> {
    // user input
}

fn main() {
    let result = rupple();
    if let Some(result) = result {
        println!(\"{:?}\", result);
    }
}
";

const BASE_CONTENTS_NO_OUTPUT: &str = "
fn rupple() {
    // user input
}

fn main() {
    rupple();
}";

fn format(mut input: String, with_output: bool) -> String {
    let base = if with_output {
        BASE_CONTENTS
    } else {
        BASE_CONTENTS_NO_OUTPUT
    };

    if with_output {
        if !input.ends_with(";") {
            let mut lines: Vec<String> = input.split(";").map(|f| f.to_string()).collect();
            let last = lines.last_mut().unwrap();
            *last = format!("Some(Box::new({}))", last);
            input = lines.join(";");
        } else {
            input += "None";
        }
    } else {
        if !input.ends_with(";") {
            input += ";";
        }
    }

    base.replace("// user input", &input)
}

/// Returns success
fn run(
    input: String,
    code_path: &PathBuf,
    exe_path: &PathBuf,
    modified: bool,
    with_output: bool,
) -> bool {
    // write file
    let formatted_file_contents = format(input.clone(), with_output);
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
            if with_output {
                // retry compiling, without output
                return run(input, code_path, exe_path, modified, false);
            }

            stdout().lock().write_all(&compile_process.stderr).unwrap();
            return false;
        }
    }
    // color stdout to make output pop
    execute!(stdout(), SetForegroundColor(crossterm::style::Color::Blue)).unwrap();

    // run
    Command::new(exe_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
}

const HELP: &str = "/help - prints help
/clear - clears repl
/exit - quits repl
/debug - prints stored repl data";

fn main() {
    let temp_dir = tempdir::TempDir::new("rupple").expect("couldn't create temp dir");

    let code_path = temp_dir.path().join("main.rs");
    // we use .exe for all OSes, since for windows it is required, and for linux file extension doesn't matter
    // so its fine
    let exe_path = temp_dir.path().join("main.exe");

    let mut stdout = stdout();
    queue!(stdout, SetForegroundColor(crossterm::style::Color::Green)).unwrap();
    println!("[rupple {}]", env!("CARGO_PKG_VERSION"));
    queue!(
        stdout,
        SetForegroundColor(crossterm::style::Color::DarkGrey)
    )
    .unwrap();
    println!("do /help for list of commands, or just start typing code!");

    let mut current_file_contents = String::new();

    loop {
        let mut modified_file_contents = current_file_contents.clone();
        queue!(stdout, SetForegroundColor(crossterm::style::Color::Green)).unwrap();
        print!("> ");
        execute!(stdout, ResetColor).unwrap();

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim();

        if buf.starts_with("/") {
            match buf {
                "/help" => {
                    println!("{HELP}")
                }
                "/clear" | "/reset" => {
                    current_file_contents = String::new();
                    std::fs::remove_file(&exe_path).unwrap();
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
            if !modified_file_contents.is_empty() && !modified_file_contents.ends_with(";") {
                modified_file_contents += ";";
            }
            modified_file_contents += buf;

            let success = run(
                modified_file_contents.clone(),
                &code_path,
                &exe_path,
                !buf.is_empty(),
                true,
            );
            queue!(stdout, ResetColor).unwrap();

            if success {
                // only save changes if it compiled successfully
                current_file_contents = modified_file_contents;
            }
        }
    }
}
