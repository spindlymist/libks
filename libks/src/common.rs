#[cfg(not(feature = "fractional-coords"))]
pub type WorldGridScalar = i32;

#[cfg(feature = "fractional-coords")]
pub type WorldGridScalar = f64;

pub type ScreenCoord = (WorldGridScalar, WorldGridScalar);

pub fn parse_xy(s: &str) -> Option<ScreenCoord> {
    let (x_str, y_str) =
        s.strip_prefix('x')?
        .split_once('y')?;
    
    let x;
    let y;

    #[cfg(not(feature = "fractional-coords"))]
    {
        x = str::parse::<i32>(x_str).ok()?;
        y = str::parse::<i32>(y_str).ok()?;
    }
    
    #[cfg(feature = "fractional-coords")]
    {
        x = 
            if x_str.contains(['.', 'e', 'E']) {
                str::parse::<f32>(x_str).ok()? as f64
            }
            else {
                str::parse::<i32>(x_str).ok()? as f64
            };
        
        y =
            if y_str.contains(['.', 'e', 'E']) {
                str::parse::<f32>(y_str).ok()? as f64
            }
            else {
                str::parse::<i32>(y_str).ok()? as f64
            };
    }
    
    Some((x, y))
}
