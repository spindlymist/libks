pub fn trimmed_range_start(s: &str) -> usize {
    for (i, byte) in s.as_bytes().iter().enumerate() {
        if *byte != b' ' {
            return i;
        }
    }
    s.len()
}

pub fn trimmed_range_end(s: &str) -> usize {
    for (i, byte) in s.as_bytes().iter().enumerate().rev() {
        if *byte != b' ' {       
            return i + 1;
        }
    }
    0
}

pub fn trimmed_range(s: &str) -> (usize, usize) {
    let start = trimmed_range_start(s);
    if start == s.len() {
        return (s.len(), s.len());
    }

    let end = trimmed_range_end(s);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trimmed_range_works() {
        let s = "     hello world     ";
        assert_eq!(trimmed_range(s), (5, 16));
    }

    #[test]
    fn trimmed_range_works_with_nothing_to_trim() {
        let s = "hello world";
        assert_eq!(trimmed_range(s), (0, s.len()));
    }

    #[test]
    fn trimmed_range_works_with_pure_whitespace() {
        let s = "     ";
        assert_eq!(trimmed_range(s), (s.len(), s.len()));
    }

    #[test]
    fn trimmed_range_works_with_empty_string() {
        let s = "";
        assert_eq!(trimmed_range(s), (0, 0));
    }
}
