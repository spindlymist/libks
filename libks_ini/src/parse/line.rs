use std::ops::Range;

use crate::whitespace::LineEnding;
use super::trim::trimmed_range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub full: Range<usize>,
    pub trimmed: Range<usize>,
    pub ws_before: Range<usize>,
    pub ws_after: Range<usize>,
    pub next_offset: Option<usize>,
    pub line_ending: LineEnding,
}

pub fn next_line(s: &str, offset: usize) -> Line {
    let end: usize;
    let line_ending: LineEnding;
    let next_offset: Option<usize>;
    
    if let Some(mut i) = memchr::memchr2(b'\r', b'\n', s[offset..].as_bytes()) {
        i += offset;
        end = i;
        match s.as_bytes()[i] {
            b'\r' if s.as_bytes().get(i + 1) == Some(&b'\n') => {
                line_ending = LineEnding::CrLf;
                next_offset = Some(i + 2);
            }
            b'\r' => {
                line_ending = LineEnding::Cr;
                next_offset = Some(i + 1);
            }
            b'\n' => {
                line_ending = LineEnding::Lf;
                next_offset = Some(i + 1);
            }
            _ => unreachable!(),
        }
    }
    else {
        end = s.len();
        line_ending = LineEnding::None;
        next_offset = None;
    }

    let (start_trimmed, end_trimmed) = trimmed_range(&s[offset..end]);

    Line {
        full: offset..end,
        trimmed: (start_trimmed + offset)..(end_trimmed + offset),
        ws_before: offset..(start_trimmed + offset),
        ws_after: (end_trimmed + offset)..end,
        next_offset,
        line_ending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HW_LEN: usize = "hello world".len();
    
    #[test]
    fn next_line_works_with_newline_only() {
        let s = "hello world\ngoodbye";
        assert_eq!(next_line(s, 0), Line {
            full: 0..HW_LEN,
            trimmed: 0..HW_LEN,
            ws_before: 0..0,
            ws_after: HW_LEN..HW_LEN,
            next_offset: Some(HW_LEN + 1),
            line_ending: LineEnding::Lf,
        });
    }

    #[test]
    fn next_line_works_with_carriage_return_only() {
        let s = "hello world\rgoodbye";
        assert_eq!(next_line(s, 0), Line {
            full: 0..HW_LEN,
            trimmed: 0..HW_LEN,
            ws_before: 0..0,
            ws_after: HW_LEN..HW_LEN,
            next_offset: Some(HW_LEN + 1),
            line_ending: LineEnding::Cr,
        });
    }

    #[test]
    fn next_line_works_with_crlf() {
        let s = "hello world\r\ngoodbye";
        assert_eq!(next_line(s, 0), Line {
            full: 0..HW_LEN,
            trimmed: 0..HW_LEN,
            ws_before: 0..0,
            ws_after: HW_LEN..HW_LEN,
            next_offset: Some(HW_LEN + 2),
            line_ending: LineEnding::CrLf,
        });
    }

    #[test]
    fn next_line_works_at_end_of_string() {
        let s = "hello world";
        assert_eq!(next_line(s, 0), Line {
            full: 0..HW_LEN,
            trimmed: 0..HW_LEN,
            ws_before: 0..0,
            ws_after: HW_LEN..HW_LEN,
            next_offset: None,
            line_ending: LineEnding::None,
        });
    }

    #[test]
    fn next_line_works_at_end_of_string_with_trailing_newline() {
        let s = "hello world\n";
        assert_eq!(next_line(s, 0), Line {
            full: 0..HW_LEN,
            trimmed: 0..HW_LEN,
            ws_before: 0..0,
            ws_after: HW_LEN..HW_LEN,
            next_offset: Some(HW_LEN + 1),
            line_ending: LineEnding::Lf,
        });
    }

    #[test]
    fn next_line_trims_correctly() {
        let s = "    hello world    \ngoodbye";
        
        let line_len = "    hello world    ".len();
        let pad_len = "    ".len();
        
        assert_eq!(next_line(s, 0), Line {
            full: 0..line_len,
            trimmed: pad_len..line_len - pad_len,
            ws_before: 0..pad_len,
            ws_after: line_len - pad_len..line_len,
            next_offset: Some(line_len + 1),
            line_ending: LineEnding::Lf,
        });
    }
}
