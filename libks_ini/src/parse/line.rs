use std::cmp::min;

use memchr::{memchr2, memchr3};

use super::trim::trimmed_range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub start_trimmed: usize,
    pub end_trimmed: usize,
    pub eq: Option<usize>,
    pub end: usize,
    pub start_next: usize,
}

impl Line {
    pub fn offset(self, offset: usize) -> Line {
        Line {
            start_trimmed: self.start_trimmed + offset,
            end_trimmed: self.end_trimmed + offset,
            eq: self.eq.map(|eq| eq + offset),
            end: self.end + offset,
            start_next: self.start_next + offset,
        }
    }
}

pub fn next_line(s: &str) -> Option<Line> {
    if s.is_empty() { return None }

    let (end, line_ending, eq) = match memchr3(b'\r', b'\n', b'=', s.as_bytes()) {
        Some(i) => match s.as_bytes()[i] {
            b'=' => {
                let eq = Some(i);
                let rest = &s[i + 1..];
                match memchr2(b'\r', b'\n', rest.as_bytes()) {
                    Some(j) => {
                        let byte = rest.as_bytes()[j];
                        (i + 1 + j, Some(byte), eq)
                    },
                    None => (s.len(), None, eq),
                }
            },
            byte => (i, Some(byte), None),
        },
        None => (s.len(), None, None),
    };

    let start_next =
        if line_ending == Some(b'\r')
            && end + 1 < s.len()
            && s.as_bytes()[end + 1] == b'\n'
        {
            min(end + 2, s.len())
        }
        else {
            min(end + 1, s.len())
        };

    let (start_trimmed, end_trimmed) = trimmed_range(&s[..end]);

    Some(Line {
        start_trimmed,
        eq,
        end_trimmed,
        end,
        start_next
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_line_works_with_newline_only() {
        let s = "hello world\r\ngoodbye";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 0,
            end_trimmed: 11,
            eq: None,
            end: 11,
            start_next: 13,
        }));
    }

    #[test]
    fn next_line_works_with_carriage_return_only() {
        let s = "hello world\r\ngoodbye";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 0,
            end_trimmed: 11,
            eq: None,
            end: 11,
            start_next: 13,
        }));
    }

    #[test]
    fn next_line_works_with_crlf() {
        let s = "hello world\r\ngoodbye";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 0,
            end_trimmed: 11,
            eq: None,
            end: 11,
            start_next: 13,
        }));
    }

    #[test]
    fn next_line_works_at_end_of_string() {
        let s = "hello world";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 0,
            end_trimmed: 11,
            eq: None,
            end: 11,
            start_next: s.len(),
        }));
    }

    #[test]
    fn next_line_works_at_end_of_string_with_trailing_newline() {
        let s = "hello world\n";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 0,
            end_trimmed: 11,
            eq: None,
            end: 11,
            start_next: s.len(),
        }));
    }

    #[test]
    fn next_line_trims_correctly() {
        let s = "    hello world    \ngoodbye";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 4,
            end_trimmed: 15,
            eq: None,
            end: 19,
            start_next: 20,
        }));
    }

    #[test]
    fn next_line_locates_equal_sign() {
        let s = "hello = world\ngoodbye";
        assert_eq!(next_line(s), Some(Line {
            start_trimmed: 0,
            end_trimmed: 13,
            eq: Some(6),
            end: 13,
            start_next: 14,
        }));
    }
}
