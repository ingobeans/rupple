use crate::parser::{parse_input, ParseResult};

#[test]
fn test_parser() {
    let tests = [
        (
            ParseResult::AllGood,
            "fn main() {
                
            }",
        ),
        (
            ParseResult::AllGood,
            "fn main() {
                // if 0 == {
            }",
        ),
        (ParseResult::RequiresSemicolon, "let g = 0"),
        (ParseResult::RequiresSemicolon, "let soooo = 'no'"),
        (ParseResult::RequiresSemicolon, "let r=2"),
        (ParseResult::Incomplete, "fn main() {"),
        (ParseResult::Incomplete, "let b = 'aa"),
        (
            ParseResult::RequiresSemicolon,
            "let c = \"a
            b
            c
            // \"",
        ),
        (ParseResult::Incomplete, "fn main() {//}"),
    ];
    for (expected, code) in tests {
        let result = parse_input(code);
        if result != expected {
            panic!(
                "Parser fail! Actual result '{:?}' != '{:?}'.\nCode: {:?}",
                result, expected, code
            );
        }
    }
}
