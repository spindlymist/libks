#[cfg(feature = "decimal")]
use rust_decimal::prelude::*;

#[cfg(not(feature = "fractional-coords"))]
pub type WorldGridScalar = i64;

#[cfg(all(feature = "fractional-coords", not(feature = "decimal")))]
pub type WorldGridScalar = f64;

#[cfg(feature = "decimal")]
pub type WorldGridScalar = Decimal;

pub type ScreenCoord = (WorldGridScalar, WorldGridScalar);

pub fn parse_xy(s: &str) -> Option<ScreenCoord> {
    let (x, y) =
        s.strip_prefix('x')?
        .split_once('y')?;

    let Ok(x) = str::parse::<WorldGridScalar>(x) else { return None };
    let Ok(y) = str::parse::<WorldGridScalar>(y) else { return None };

    Some((x, y))
}
