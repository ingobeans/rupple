pub enum ParseResult {
    /// No problems found
    AllGood,
    /// Input requires to be suffixed with a semicolon. Happens on ex. `let a = 5`.
    RequiresSemicolon,
    /// Input is incomplete, i.e. a closure or string has been opened with being closed
    Incomplete,
}

/// Pops chars off a Vec<char> until a stop char is hit. Optionally backslash escaped.
/// Returns whether a stop was hit before EOF.
fn pop_until(chars: &mut Vec<char>, stop: char, backslash_escaped: bool) -> bool {
    while let Some(popped) = chars.pop() {
        if backslash_escaped && popped == '\\' {
            chars.pop();
            continue;
        }
        if popped == stop {
            return true;
        }
    }
    false
}

/// Parses user input code. Returns [ParseResult].
/// Used to identify ex. whether input requires to be suffixed with a semicolon, or input is incomplete.
pub fn parse_input(input: &str) -> ParseResult {
    let mut chars = input.chars().rev().collect::<Vec<char>>();

    let mut keywords: Vec<String> = vec![String::new()];
    let mut opened_closures: usize = 0;

    while let Some(char) = chars.pop() {
        match char {
            '\\' => {
                chars.pop();
            }
            '\'' | '"' => {
                let stopped = pop_until(&mut chars, char, true);
                if !stopped {
                    return ParseResult::Incomplete;
                }
                *keywords.last_mut().unwrap() = String::from("value");
            }
            '/' => {
                let next = chars.pop().unwrap_or(' ');
                if next == '/' {
                    pop_until(&mut chars, '\n', false);
                } else {
                    chars.push(next);
                }
            }
            '{' => {
                opened_closures += 1;
            }
            '}' => {
                opened_closures = opened_closures.saturating_sub(1);
            }
            ' ' => {
                keywords.push(String::new());
            }
            _ => {
                keywords.last_mut().unwrap().push(char);
            }
        }
    }
    if opened_closures > 0 {
        ParseResult::Incomplete
    } else {
        let length = keywords.len();
        // if theres at least 4 keywords, and
        // the fourth last keyword is "let" and the second last keyword is "=",
        // then that means we have a `let x = y` pattern,
        // and as such, a semicolon is required
        if length >= 4 && keywords[length - 4].trim() == "let" && keywords[length - 2].trim() == "="
        {
            ParseResult::RequiresSemicolon
        } else {
            ParseResult::AllGood
        }
    }
}
