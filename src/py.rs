//! Python 3.9's numbers, for a port that has to write what the Python wrote:
//! float repr, format(g), round(), math.hypot and libm's sin and cos.
//! From lasermade-tools (github.com/Gernreich/lasermade-tools), src/lib.rs.
#![allow(dead_code)]
use std::hint::black_box;

/// repr(x) for a float: the shortest digits that read back as x, rounded
/// half to even where two candidates tie (Rust's shortest form rounds the
/// tie up), in Python's layout: 1e-05, 1e+16, 600.0.
pub fn repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    // repr is the closest string of the shortest length that reads back,
    // ties to even. Rust's {:e} has that length but breaks ties upwards
    // (2**-25 ends ...53125 at 17 digits: repr 312, {:e} 313), so take the
    // exact value correctly rounded to that length. That one can fail to
    // read back beside a power of two, whose lower gap is half the upper:
    // 2**-44 rounds to ...801 at 16 digits, which is another double, and
    // there the shortest form, ...802, is repr's.
    let short = format!("{:e}", x);
    let nd = short.split('e').next().unwrap().chars().filter(|c| c.is_ascii_digit()).count();
    let even = format!("{:.*e}", nd - 1, x);
    let s = if even.parse::<f64>() == Ok(x) { even } else { short };
    let (mant, exp) = s.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let sign = if mant.starts_with('-') { "-" } else { "" };
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    if !(-4..16).contains(&exp) {
        let m = if digits.len() > 1 {
            format!("{}.{}", &digits[..1], &digits[1..])
        } else {
            digits
        };
        format!("{sign}{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs())
    } else if exp >= 0 {
        let e = exp as usize;
        if digits.len() > e + 1 {
            format!("{sign}{}.{}", &digits[..e + 1], &digits[e + 1..])
        } else {
            format!("{sign}{}{}.0", digits, "0".repeat(e + 1 - digits.len()))
        }
    } else {
        format!("{sign}0.{}{}", "0".repeat((-exp - 1) as usize), digits)
    }
}

/// f'{x:g}': six significant digits, trailing zeros dropped, the
/// exponent form below 1e-4 and from 1e6.
pub fn g(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    let s = format!("{:.5e}", x);
    let (m, e) = s.split_once('e').unwrap();
    let exp: i32 = e.parse().unwrap();
    let strip = |t: &str| -> String {
        if t.contains('.') {
            t.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            t.to_string()
        }
    };
    if (-4..6).contains(&exp) {
        strip(&format!("{:.*}", (5 - exp) as usize, x))
    } else {
        format!("{}e{}{:02}", strip(m), if exp < 0 { '-' } else { '+' }, exp.abs())
    }
}

/// f'{x:.{p}e}': Python's exponent has a sign and at least two digits.
pub fn e(x: f64, p: usize) -> String {
    let s = format!("{:.*e}", p, x);
    let (m, e) = s.split_once('e').unwrap();
    let exp: i32 = e.parse().unwrap();
    format!("{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs())
}

/// math.degrees
/// base64.b64encode(b).decode()
pub fn b64(b: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            s.push(if k <= c.len() { A[(n >> (18 - 6 * k) & 63) as usize] as char } else { '=' });
        }
    }
    s
}

/// math.log2: libm's, behind black_box so nothing is folded
pub fn log2(x: f64) -> f64 {
    extern "C" {
        fn log2(x: f64) -> f64;
    }
    let f: unsafe extern "C" fn(f64) -> f64 = log2;
    unsafe { black_box(f)(black_box(x)) }
}

pub fn degrees(x: f64) -> f64 {
    x * (180.0 / std::f64::consts::PI)
}

/// round(x): to the nearest integer, halves to even.
pub fn round(x: f64) -> i64 {
    x.round_ties_even() as i64
}

/// round(x, dp): correctly rounded to dp decimals, as Python does it.
pub fn round_dp(x: f64, dp: usize) -> f64 {
    format!("{:.*}", dp, x).parse().unwrap()
}

/// CPython 3.9's vector_norm, which math.hypot and math.dist use: scaled
/// by the largest component, summed with a compensation term. Not the
/// same bits as f64::hypot.
fn vector_norm(v: &[f64]) -> f64 {
    let mut max = 0.0f64;
    let mut nan = false;
    for &x in v {
        let x = x.abs();
        nan |= x.is_nan();
        if x > max {
            max = x;
        }
    }
    if max.is_infinite() {
        return max;
    }
    if nan {
        return f64::NAN;
    }
    if max == 0.0 || v.len() <= 1 {
        return max;
    }
    let (mut csum, mut frac) = (1.0f64, 0.0f64);
    for &x0 in v {
        let mut x = x0.abs();
        x /= max;
        x *= x;
        let old = csum;
        csum += x;
        frac += (old - csum) + x;
    }
    max * (csum - 1.0 + frac).sqrt()
}

/// math.hypot(x, y)
pub fn hypot(x: f64, y: f64) -> f64 {
    vector_norm(&[x, y])
}

/// math.dist(p, q) in the plane
pub fn dist(p: (f64, f64), q: (f64, f64)) -> f64 {
    vector_norm(&[p.0 - q.0, p.1 - q.1])
}

type F1 = unsafe extern "C" fn(f64) -> f64;
extern "C" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn tan(x: f64) -> f64;
    fn acos(x: f64) -> f64;
    fn pow(x: f64, y: f64) -> f64;
    fn atan2(y: f64, x: f64) -> f64;
    fn fmod(x: f64, y: f64) -> f64;
}
fn call(f: F1, x: f64) -> f64 {
    unsafe { black_box(f)(x) }
}
// Each through a pointer the compiler cannot see through, so a sin and a
// cos of one angle stay two calls, as Python makes them, and are not
// merged into one sincos that rounds differently.
pub fn msin(x: f64) -> f64 {
    call(sin, x)
}
pub fn mcos(x: f64) -> f64 {
    call(cos, x)
}
pub fn mtan(x: f64) -> f64 {
    call(tan, x)
}
pub fn macos(x: f64) -> f64 {
    call(acos, x)
}
/// x ** y for floats: libm's pow, which is not always x * x
pub fn powf(x: f64, y: f64) -> f64 {
    let f: unsafe extern "C" fn(f64, f64) -> f64 = pow;
    unsafe { black_box(f)(x, black_box(y)) }
}
/// math.atan2: libm's
pub fn matan2(y: f64, x: f64) -> f64 {
    let f: unsafe extern "C" fn(f64, f64) -> f64 = atan2;
    unsafe { black_box(f)(y, black_box(x)) }
}
/// math.remainder: CPython's own m_remainder, not libm's, for finite x and
/// y != 0 (the only case asked of it here)
pub fn remainder(x: f64, y: f64) -> f64 {
    let f: unsafe extern "C" fn(f64, f64) -> f64 = fmod;
    let (absx, absy) = (x.abs(), y.abs());
    let m = unsafe { black_box(f)(absx, absy) };
    let c = absy - m;
    let r = if m < c {
        m
    } else if m > c {
        -c
    } else {
        m - 2.0 * unsafe { black_box(f)(0.5 * (absx - m), absy) }
    };
    1.0f64.copysign(x) * r
}
/// math.radians
pub fn radians(x: f64) -> f64 {
    x * (std::f64::consts::PI / 180.0)
}

/// max(a, b) and min(a, b): the first unless the second is strictly
/// beyond it, which is also what they do with a NaN.
pub fn max(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}
pub fn min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// float(s): surrounding whitespace allowed, as Python allows it.
pub fn float(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() || t.contains('_') && !t.chars().all(|c| c.is_ascii_digit() || "._-+eE".contains(c)) {
        return None;
    }
    t.replace('_', "").parse().ok()
}

/// repr(s) for a str: single quotes unless the text holds one and no
/// double quote; backslash escapes for the quote, backslash and controls.
pub fn str_repr(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut o = String::new();
    o.push(q);
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if c == q => {
                o.push('\\');
                o.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => o.push_str(&format!("\\x{:02x}", c as u32)),
            c if (0x80..0xa0).contains(&(c as u32)) => o.push_str(&format!("\\x{:02x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push(q);
    o
}

/// str(list_of_str): ['a', 'b']
pub fn list_repr<S: AsRef<str>>(xs: &[S]) -> String {
    let v: Vec<String> = xs.iter().map(|x| str_repr(x.as_ref())).collect();
    format!("[{}]", v.join(", "))
}

/// str(list_of_int): [1, 2]
pub fn ints_repr<T: std::fmt::Display>(xs: &[T]) -> String {
    let v: Vec<String> = xs.iter().map(|x| x.to_string()).collect();
    format!("[{}]", v.join(", "))
}

/// s[:n], counted in characters
pub fn head(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}
