/// Whether a user's input is incomplete or not.
/// For instance, true if the user opens a closure that isn't closed,
/// or declares a string without terminating quote.
pub fn is_input_incomplete(input: &str) -> bool {
    let mut chars = input.chars().rev().collect::<Vec<char>>();

    let mut enclosed: Option<char> = None;

    let mut opened_closures: usize = 0;

    while let Some(char) = chars.pop() {
        match char {
            '\\' => {
                chars.pop();
            }
            '/' if enclosed.is_none() => {
                let next = chars.pop().unwrap_or(' ');
                if next == '/' {
                    // pop chars until new line
                    while let Some(popped) = chars.pop() {
                        if popped == '\n' {
                            break;
                        }
                    }
                } else {
                    chars.push(next);
                }
            }
            '\'' | '"' => {
                if let Some(enclosure_char) = enclosed {
                    if enclosure_char == char {
                        enclosed = None;
                    }
                } else {
                    enclosed = Some(char);
                }
            }
            '{' if enclosed.is_none() => {
                opened_closures += 1;
            }
            '}' if enclosed.is_none() => {
                opened_closures = opened_closures.saturating_sub(1);
            }
            _ => {}
        }
    }
    opened_closures > 0
}
