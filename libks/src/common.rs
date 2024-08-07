pub fn parse_xy(s: &str) -> Option<(i64, i64)> {
    let (x, y) =
        s.strip_prefix('x')?
        .split_once('y')?;
    
    let Ok(x) = str::parse::<i64>(x) else { return None };
    let Ok(y) = str::parse::<i64>(y) else { return None };

    Some((x, y))
}
