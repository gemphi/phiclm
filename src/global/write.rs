//! BLIB's writers' arithmetic: a string out of the store, a capital, a
//! number in a field, digits in a radix, and two strings compared.

use crate::machine::Machine;

/// A length-prefixed string read out of the store.
pub(super) fn text(m: &Machine, at: i64) -> String {
    let len = m.mem.get(at as usize).copied().unwrap_or(0).max(0) as usize;
    (0..len)
        .filter_map(|i| m.mem.get(at as usize + 1 + i))
        .filter_map(|w| char::from_u32(*w as u32))
        .collect()
}

/// `capitalch` (`blib.b:864`): a lower-case letter raised, anything else kept.
pub(super) fn capital(c: i64) -> i64 {
    match c {
        0x61..=0x7a => c - 0x20,
        _ => c,
    }
}

/// `writedz` (`blib.b:587`): `n` in a field of `d`, padded with spaces or
/// zeroes; the sign goes before zeroes and after spaces.
pub(super) fn decimal(out: &mut String, n: i64, d: i64, zeroes: bool) {
    let digits = n.unsigned_abs().to_string();
    let neg = n < 0;
    let pad = (d - neg as i64 - digits.len() as i64).max(0) as usize;
    let fill = match zeroes {
        true => '0',
        false => ' ',
    };
    match (neg, zeroes) {
        (true, true) => {
            out.push('-');
            out.extend(std::iter::repeat(fill).take(pad));
        }
        (true, false) => {
            out.extend(std::iter::repeat(fill).take(pad));
            out.push('-');
        }
        (false, _) => out.extend(std::iter::repeat(fill).take(pad)),
    }
    out.push_str(&digits);
}

/// `writehex`/`writeoct`/`writebin` (`blib.b:607–623`): exactly `d` digits
/// (at least one) of `bits` bits each, high first, in capitals.
pub(super) fn radix(out: &mut String, n: i64, d: i64, bits: u32) {
    const DIGIT: &[u8; 16] = b"0123456789ABCDEF";
    for i in (0..d.max(1) as u32).rev() {
        let shift = i * bits;
        let k = match shift < 64 {
            true => ((n as u64) >> shift) & ((1 << bits) - 1),
            false => 0,
        };
        out.push(DIGIT[k as usize] as char);
    }
}

/// `compstring` (`blib.b:868`): case-blind, then the shorter first.
pub(super) fn compare(m: &Machine, a: i64, b: i64) -> i64 {
    let (a, b): (Vec<i64>, Vec<i64>) = (text(m, a).chars().map(|c| c as i64).collect(), text(m, b).chars().map(|c| c as i64).collect());
    match a.iter().zip(&b).map(|(x, y)| capital(*x) - capital(*y)).find(|r| *r != 0) {
        Some(r) => r,
        None => (a.len() as i64 - b.len() as i64).signum(),
    }
}
