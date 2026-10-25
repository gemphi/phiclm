//! The global vector's native residents — BLIB's heart. In cintsys the
//! library lives in global slots the compiler never owns; here the first
//! slots hold negative entries the machine dispatches to Rust. Output goes
//! to the machine's `out` buffer, so the judge can read what a program said.

use crate::machine::Machine;

/// The native globals, in slot order. `writef`'s `%n %s %c %x` are the
/// escapes the `com/` sources lean on most.
pub const NATIVE: &[&str] = &["writef", "writes", "newline", "writen", "wrch"];

/// A length-prefixed string read out of the store.
fn text(m: &Machine, at: i64) -> String {
    let len = m.mem.get(at as usize).copied().unwrap_or(0).max(0) as usize;
    (0..len)
        .filter_map(|i| m.mem.get(at as usize + 1 + i))
        .filter_map(|w| char::from_u32(*w as u32))
        .collect()
}

/// Runs native entry `f` (negative) on `arg`; the result is its value.
pub fn native(m: &mut Machine, f: i64, arg: &[i64]) -> Result<i64, String> {
    match (-f - 1) as usize {
        0 => {
            // writef(format, a, b, …)
            let fmt = text(m, *arg.first().ok_or("writef with no format")?);
            let mut used = 1;
            let mut c = fmt.chars().peekable();
            while let Some(ch) = c.next() {
                if ch != '%' {
                    m.out.push(ch);
                    continue;
                }
                let kind = c.next().unwrap_or('%');
                let value = arg.get(used).copied().unwrap_or(0);
                match kind.to_ascii_lowercase() {
                    'n' | 'i' | 'd' => {
                        used += 1;
                        m.out.push_str(&value.to_string());
                    }
                    's' => {
                        used += 1;
                        let s = text(m, value);
                        m.out.push_str(&s);
                    }
                    'c' => {
                        used += 1;
                        if let Some(ch) = char::from_u32(value as u32) {
                            m.out.push(ch);
                        }
                    }
                    'x' => {
                        used += 1;
                        m.out.push_str(&format!("{value:x}"));
                    }
                    other => m.out.push(other),
                }
            }
            Ok(0)
        }
        1 => {
            let s = text(m, *arg.first().ok_or("writes with nothing to write")?);
            m.out.push_str(&s);
            Ok(0)
        }
        2 => {
            m.out.push('\n');
            Ok(0)
        }
        3 => {
            m.out.push_str(&arg.first().copied().unwrap_or(0).to_string());
            Ok(0)
        }
        4 => {
            if let Some(ch) = char::from_u32(arg.first().copied().unwrap_or(0) as u32) {
                m.out.push(ch);
            }
            Ok(0)
        }
        other => Err(format!("no native {other}")),
    }
}
