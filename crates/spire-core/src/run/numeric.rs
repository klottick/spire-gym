//! Shared source rounding laws (M29 design §6.2). JDK `Math.round(float)` and libGDX
//! `MathUtils.round(float)` round differently; each caller names the one its source uses.

/// JDK 7+ `Math.round(float)`: the exact floor(x + ½) of the float value (not a float add of
/// 0.5f, which mis-rounds 0.49999997f and odd values at or above 2^23), NaN 0, saturating.
pub(crate) fn java_round_f32(value:f32)->i32{
    if value.is_nan(){return 0}
    // binary64 holds every binary32 value plus ½ exactly, so the floor is exact.
    let rounded=(f64::from(value)+0.5).floor();
    if rounded>=f64::from(i32::MAX){i32::MAX}else if rounded<=f64::from(i32::MIN){i32::MIN}else{rounded as i32}
}

/// libGDX `MathUtils.round(float)`: `int(binary64(x + 16384.5)) − 16384` (economy research :45-50).
pub(crate) fn libgdx_round(value:f32)->u32{
    ((f64::from(value)+16384.5) as i64-16384).clamp(0,i64::from(u32::MAX)) as u32
}
