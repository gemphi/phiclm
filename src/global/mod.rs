//! The global vector's native residents — BLIB's heart. In cintsys the
//! library lives in global slots the compiler never owns; here the first
//! slots hold negative entries the machine dispatches to Rust. Output goes
//! to the machine's `out` buffer, so the judge can read what a program said.
//! Each routine is read from `sysb/blib.b` (lines cited) and keeps its
//! behaviour, down to `writehex` writing capitals and exactly `d` digits.

use crate::machine::Machine;
use format::format;
use read::{rdch, readn};
use slots::Native;
use write::{capital, compare, decimal, radix, text};

mod format;
mod read;
mod slots;
mod stream;
mod write;

pub use slots::NATIVE;
pub(crate) use slots::{control, Control};

/// `result2`, BLIB's second result (`muldiv`'s remainder, `readn`'s
/// success), lives in the slot after the natives; the program's own
/// globals start after it.
pub const RESULT2: usize = NATIVE.len();
pub const USER: usize = RESULT2 + 1;

/// `GET "libhdr"` declares these (`g/libhdr.h:216–225`). A byte is a
/// word here and a word is 64 bits, so `bytesperword` is 1 while
/// `bitsperbyte` stays 8, the width a character is masked to.
pub const LIBHDR: &[(&str, i64)] = &[
    ("ug", 200),
    ("bytesperword", 1),
    ("bitsperbyte", 8),
    ("bitsperword", 64),
    ("mcaddrinc", 1),
    ("minint", i64::MIN),
    ("maxint", i64::MAX),
    ("endstreamch", -1),
];

/// Runs native entry `f` (negative) on `arg`; the result is its value.
pub fn native(m: &mut Machine, f: i64, arg: &[i64]) -> Result<i64, String> {
    let a = |i: usize| arg.get(i).copied().unwrap_or(0);
    let which = slots::which(f)?;
    match which {
        Native::Writef => {
            arg.first().ok_or("writef with no format")?;
            let mut next = 1;
            format(m, a(0), arg, &mut next);
        }
        Native::Writes => {
            let s = text(m, *arg.first().ok_or("writes with nothing to write")?);
            m.out.push_str(&s);
        }
        Native::Newline => m.out.push('\n'),
        Native::Writen => decimal(&mut m.out, a(0), 0, false),
        Native::Wrch => m.out.extend(char::from_u32(a(0) as u32)),
        Native::Writed => decimal(&mut m.out, a(0), a(1), false),
        Native::Writez => decimal(&mut m.out, a(0), a(1), true),
        Native::Writeu => {
            let u = (a(0) as u64).to_string();
            m.out.extend(std::iter::repeat(' ').take((a(1) - u.len() as i64).max(0) as usize));
            m.out.push_str(&u);
        }
        Native::Writehex => radix(&mut m.out, a(0), a(1), 4),
        Native::Writeoct => radix(&mut m.out, a(0), a(1), 3),
        Native::Writebin => radix(&mut m.out, a(0), a(1), 1),
        Native::Capitalch => return Ok(capital(a(0))),
        Native::Compch => return Ok(capital(a(0)) - capital(a(1))),
        Native::Compstring => return Ok(compare(m, a(0), a(1))),
        Native::Getvec => return Ok(m.getvec(a(0))),
        Native::Freevec => m.freevec(a(0)),
        Native::Writet => {
            let s = text(m, a(0));
            let pad = (a(1) - s.chars().count() as i64).max(0) as usize;
            m.out.push_str(&s);
            m.out.extend(std::iter::repeat(' ').take(pad));
        }
        // A character is a word here, so a Unicode code is written whole.
        Native::Codewrch => m.out.extend(char::from_u32(a(0) as u32)),
        Native::Rdch => return Ok(rdch(m)),
        Native::Unrdch => {
            m.read = m.read.saturating_sub(1);
            return Ok(-1);
        }
        Native::Readn => return Ok(readn(m)),
        Native::Randno => {
            // `blib.b:839`: 1..=upb from the linear congruence.
            m.seed = m.seed.wrapping_mul(2147001325).wrapping_add(715136305);
            let upb = a(0);
            return match upb > 0 {
                true => Ok((m.seed / 3).wrapping_abs() % upb + 1),
                false => Err(format!("randno({upb}): the bound must be positive")),
            };
        }
        Native::Setseed => return Ok(std::mem::replace(&mut m.seed, a(0))),
        Native::Muldiv => {
            // `a * b / c` without overflow in between; the remainder goes to result2.
            let (p, c) = (a(0) as i128 * a(1) as i128, a(2) as i128);
            let q = p.checked_div(c).ok_or("muldiv by zero")?;
            m.mem[RESULT2] = (p % c) as i64;
            return Ok(q as i64);
        }
        Native::Packstring => {
            // `blib.b:858`: bytes are words, so packing is a copy; the size
            // in words past the first is the length.
            let (v, s) = (a(0), a(1));
            let n = m.mem.get(v as usize).copied().unwrap_or(0) & 255;
            for i in 0..=n {
                let w = m.mem.get((v + i) as usize).copied().ok_or("packstring outside the store")?;
                *m.mem.get_mut((s + i) as usize).ok_or("packstring outside the store")? = w;
            }
            return Ok(n);
        }
        Native::Findinput => return Ok(stream::findinput(m, a(0))),
        Native::Findoutput => return Ok(stream::findoutput(m, a(0))),
        Native::Selectinput => stream::selectinput(m, a(0))?,
        Native::Selectoutput => stream::selectoutput(m, a(0))?,
        Native::Endread => stream::endread(m)?,
        Native::Endwrite => stream::endwrite(m)?,
        Native::Input => return Ok(m.cis as i64),
        Native::Output => return Ok(m.cos as i64),
        Native::Createco | Native::Deleteco | Native::Callco | Native::Resumeco | Native::Cowait | Native::Level
        | Native::Longjump | Native::Stop | Native::Abort => {
            return Err(format!("{which:?} moves the machine: the run loop does it, not a native call"));
        }
        Native::Unpackstring => {
            let (s, v) = (a(0), a(1));
            let n = m.mem.get(s as usize).copied().unwrap_or(0).max(0);
            for i in (0..=n).rev() {
                let w = m.mem.get((s + i) as usize).copied().ok_or("unpackstring outside the store")?;
                *m.mem.get_mut((v + i) as usize).ok_or("unpackstring outside the store")? = w;
            }
        }
    }
    Ok(0)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slots_and_the_routines_line_up() {
        assert_eq!(NATIVE.len(), slots::ORDER.len());
        for (name, n) in NATIVE.iter().zip(slots::ORDER) {
            assert_eq!(format!("{n:?}").to_lowercase(), *name);
        }
    }
}
