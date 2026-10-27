//! BLIB's input: `rdch` over the judge's one input string, and `readn`.

use super::RESULT2;
use crate::machine::Machine;

/// `rdch`: the next character of the input, or `endstreamch` (-1).
pub(super) fn rdch(m: &mut Machine) -> i64 {
    let c = m.input.get(m.read).copied().unwrap_or(-1);
    m.read = (m.read + 1).min(m.input.len() + 1);
    c
}

/// `readn` (`blib.b:550`): spaces, tabs and newlines skipped, a sign, the
/// digits; the character after them is given back. `result2` is 0 when a
/// number was read, -1 when none was.
pub(super) fn readn(m: &mut Machine) -> i64 {
    let (mut sum, mut neg) = (0i64, false);
    let mut ch = rdch(m);
    loop {
        match ch {
            0x30..=0x39 => break,
            0x20 | 0x09 | 0x0a => ch = rdch(m),
            0x2d | 0x2b => {
                neg = ch == 0x2d;
                ch = rdch(m);
                break;
            }
            _ => {
                m.read = m.read.saturating_sub(1);
                m.mem[RESULT2] = -1;
                return 0;
            }
        }
    }
    while (0x30..=0x39).contains(&ch) {
        sum = sum.wrapping_mul(10).wrapping_add(ch - 0x30);
        ch = rdch(m);
    }
    m.read = m.read.saturating_sub(1);
    m.mem[RESULT2] = 0;
    match neg {
        true => -sum,
        false => sum,
    }
}
