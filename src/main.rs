use std::{
    io::{Read, Write, stdin, stdout},
    path::PathBuf,
    process::{Command, Stdio},
};

const BASE_CONTENTS: &str = "
use std::fmt::Debug;
fn rupple() -> Box<dyn Debug> {
    // user input
}

fn main() {
    let result = rupple();
    println!(\"{:?}\", result);
}
";

fn format(mut input: String) -> String {
    if !input.ends_with(";") {
        let mut lines: Vec<String> = input.split(";").map(|f| f.to_string()).collect();
        let last = lines.last_mut().unwrap();
        *last = format!("Box::new({})", last);
        input = lines.join(";");
    } else {
        input += "Box::new(\"no output\")";
    }
    BASE_CONTENTS.replace("// user input", &input)
}

/// Returns success
fn run(input: String, code_path: &PathBuf, exe_path: &PathBuf) -> bool {
    // write file
    let formatted_file_contents = format(input);
    std::fs::write(code_path, formatted_file_contents).unwrap();

    // compile
    let mut compile_process = Command::new("rustc")
        .arg(code_path)
        .arg("-o")
        .arg(exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stderr = compile_process.stderr.take().unwrap();
    let compile_result = compile_process.wait().unwrap();

    if !compile_result.success() {
        let mut buf = Vec::new();
        stderr.read_to_end(&mut buf).unwrap();
        stdout().lock().write_all(&buf).unwrap();
        false
    } else {
        // run
        Command::new(exe_path)
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success()
    }
}

fn main() {
    let temp_dir = tempdir::TempDir::new("rupple").expect("couldn't create temp dir");
    let code_path = temp_dir.path().join("main.rs");
    let exe_path = temp_dir.path().join("main.exe");

    let mut current_file_contents = String::new();

    loop {
        let mut modified_file_contents = current_file_contents.clone();
        print!("> ");
        stdout().flush().unwrap();

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();

        if !modified_file_contents.ends_with(";") {
            modified_file_contents += ";";
        }
        modified_file_contents += buf.trim();

        let success = run(modified_file_contents.clone(), &code_path, &exe_path);
        if success {
            // only save changes if it compiled successfully
            current_file_contents = modified_file_contents;
        }
    }
}
