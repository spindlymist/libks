use super::{trim::trimmed_range, whitespace::LineEnding};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line<'a> {
    pub line_full: &'a str,
    pub line_trimmed: &'a str,
    pub ws_before: &'a str,
    pub ws_after: &'a str,
    pub rest: Option<&'a str>,
    pub line_ending: LineEnding,
}

pub fn next_line(s: &str) -> Line {
    let end: usize;
    let line_ending: LineEnding;
    let rest: Option<&str>;
    
    if let Some(i) = memchr::memchr2(b'\r', b'\n', s.as_bytes()) {
        end = i;
        match s.as_bytes()[i] {
            b'\r' if s.as_bytes().get(i + 1) == Some(&b'\n') => {
                line_ending = LineEnding::CrLf;
                rest = Some(&s[i + 2..]);
            }
            b'\r' => {
                line_ending = LineEnding::Cr;
                rest = Some(&s[i + 1..]);
            }
            b'\n' => {
                line_ending = LineEnding::Lf;
                rest = Some(&s[i + 1..]);
            }
            _ => unreachable!(),
        }
    }
    else {
        end = s.len();
        line_ending = LineEnding::None;
        rest = None;
    }

    let (start_trimmed, end_trimmed) = trimmed_range(&s[..end]);

    Line {
        line_full: &s[..end],
        line_trimmed: &s[start_trimmed..end_trimmed],
        ws_before: &s[..start_trimmed],
        ws_after: &s[end_trimmed..end],
        rest,
        line_ending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_line_works_with_newline_only() {
        let s = "hello world\ngoodbye";
        assert_eq!(next_line(s), Line {
            line_full: "hello world",
            line_trimmed: "hello world",
            ws_before: "",
            ws_after: "",
            rest: Some("goodbye"),
            line_ending: LineEnding::Lf,
        });
    }

    #[test]
    fn next_line_works_with_carriage_return_only() {
        let s = "hello world\rgoodbye";
        assert_eq!(next_line(s), Line {
            line_full: "hello world",
            line_trimmed: "hello world",
            ws_before: "",
            ws_after: "",
            rest: Some("goodbye"),
            line_ending: LineEnding::Cr,
        });
    }

    #[test]
    fn next_line_works_with_crlf() {
        let s = "hello world\r\ngoodbye";
        assert_eq!(next_line(s), Line {
            line_full: "hello world",
            line_trimmed: "hello world",
            ws_before: "",
            ws_after: "",
            rest: Some("goodbye"),
            line_ending: LineEnding::CrLf,
        });
    }

    #[test]
    fn next_line_works_at_end_of_string() {
        let s = "hello world";
        assert_eq!(next_line(s), Line {
            line_full: "hello world",
            line_trimmed: "hello world",
            ws_before: "",
            ws_after: "",
            rest: None,
            line_ending: LineEnding::None,
        });
    }

    #[test]
    fn next_line_works_at_end_of_string_with_trailing_newline() {
        let s = "hello world\n";
        assert_eq!(next_line(s), Line {
            line_full: "hello world",
            line_trimmed: "hello world",
            ws_before: "",
            ws_after: "",
            rest: Some(""),
            line_ending: LineEnding::Lf,
        });
    }

    #[test]
    fn next_line_trims_correctly() {
        let s = "    hello world    \ngoodbye";
        assert_eq!(next_line(s), Line {
            line_full: "    hello world    ",
            line_trimmed: "hello world",
            ws_before: "    ",
            ws_after: "    ",
            rest: Some("goodbye"),
            line_ending: LineEnding::Lf,
        });
    }
}
