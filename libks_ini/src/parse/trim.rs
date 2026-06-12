pub fn trimmed_range_start(s: &str) -> usize {
    for (i, ch) in s.char_indices() {
        if !ch.is_ascii_whitespace() {
            return i;
        }
    }
    s.len()
}

pub fn trimmed_range_end(s: &str) -> usize {
    for (i, ch) in s.char_indices().rev() {
        if !ch.is_ascii_whitespace() {    
            return i + ch.len_utf8();
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
        let (start, end) = trimmed_range(s);
        assert_eq!(&s[start..end], "hello world");
    }

    #[test]
    fn trimmed_range_works_with_nothing_to_trim() {
        let s = "hello world";
        let (start, end) = trimmed_range(s);
        assert_eq!(&s[start..end], "hello world");
    }

    #[test]
    fn trimmed_range_works_with_pure_whitespace() {
        let s = "     ";
        let (start, end) = trimmed_range(s);
        assert_eq!(&s[start..end], "");
    }

    #[test]
    fn trimmed_range_works_with_empty_string() {
        let s = "";
        let (start, end) = trimmed_range(s);
        assert_eq!(&s[start..end], "");
    }
    
    #[test]
    fn trimmed_range_works_with_multibyte_characters() {
        let s = "     今日わ     ";
        let (start, end) = trimmed_range(s);
        assert_eq!(&s[start..end], "今日わ");
    }
}
