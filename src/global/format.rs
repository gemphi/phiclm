//! `writef`'s format: BLIB's `write_format`, item by item.

use super::write::{capital, decimal, radix, text};
use crate::machine::Machine;

/// `write_format` (`blib.b:723`): `%s %c %n`, the width-taking `%i %d %x %o
/// %u %z %b %t` (a width is one digit or letter after the type, A = 10, or a
/// run of digits before it; `%n.md` scales), `%$ %+` skip an argument, `%-`
/// steps back, `%f` nests a format, `%p` plurals, anything else is itself.
pub(super) fn format(m: &mut Machine, fmt: i64, arg: &[i64], next: &mut usize) {
    let f: Vec<char> = text(m, fmt).chars().collect();
    let at = |p: usize| f.get(p).copied().unwrap_or('\0');
    let mut p = 0;
    while p < f.len() {
        let k = f[p];
        p += 1;
        if k != '%' {
            m.out.push(k);
            continue;
        }
        let (mut width, mut given, mut scale) = (0i64, false, 0i64);
        let mut ty = at(p);
        while ty.is_ascii_digit() {
            width = 10 * width + (ty as i64 - '0' as i64);
            given = true;
            p += 1;
            ty = at(p);
        }
        if given && ty == '.' {
            p += 1;
            ty = at(p);
            while ty.is_ascii_digit() {
                scale = 10 * scale + (ty as i64 - '0' as i64);
                p += 1;
                ty = at(p);
            }
        }
        p += 1;
        let value = arg.get(*next).copied().unwrap_or(0);
        let upper = ty.to_ascii_uppercase();
        match upper {
            'S' => {
                let s = text(m, value);
                m.out.push_str(&s);
                *next += 1;
            }
            'C' | '#' => {
                m.out.extend(char::from_u32(value as u32));
                *next += 1;
            }
            'N' => {
                decimal(&mut m.out, value, 0, false);
                *next += 1;
            }
            'D' if scale > 0 => {
                let unit = 10i64.pow(scale as u32);
                decimal(&mut m.out, value / unit, width - 1 - scale, false);
                m.out.push('.');
                decimal(&mut m.out, (value % unit).abs(), scale, true);
                *next += 1;
            }
            'I' | 'D' | 'X' | 'O' | 'U' | 'Z' | 'B' | 'T' => {
                if !given {
                    let w = capital(at(p) as i64);
                    width = match w {
                        0x30..=0x39 => w - 0x30,
                        _ => 10 + w - 0x41,
                    };
                    p += 1;
                }
                match upper {
                    'X' => radix(&mut m.out, value, width, 4),
                    'O' => radix(&mut m.out, value, width, 3),
                    'B' => radix(&mut m.out, value, width, 1),
                    'Z' => decimal(&mut m.out, value, width, true),
                    'U' => {
                        let u = (value as u64).to_string();
                        m.out.extend(std::iter::repeat(' ').take((width - u.len() as i64).max(0) as usize));
                        m.out.push_str(&u);
                    }
                    'T' => {
                        let s = text(m, value);
                        let pad = (width - s.chars().count() as i64).max(0) as usize;
                        m.out.push_str(&s);
                        m.out.extend(std::iter::repeat(' ').take(pad));
                    }
                    _ => decimal(&mut m.out, value, width, false),
                }
                *next += 1;
            }
            '$' | '+' => *next += 1,
            '-' => *next = next.saturating_sub(1),
            'F' => {
                *next += 1;
                format(m, value, arg, next);
            }
            'P' => {
                let plural = value != 1;
                *next += 1;
                let c = at(p);
                p += 1;
                match c {
                    '\\' => {
                        // `%p\singular\plural\` — say the one that fits.
                        let mut skipping = plural;
                        while p < f.len() {
                            let ch = f[p];
                            p += 1;
                            match (ch == '\\', skipping) {
                                (true, _) => {
                                    skipping = !skipping;
                                    if skipping == plural {
                                        break;
                                    }
                                }
                                (false, false) => m.out.push(ch),
                                (false, true) => {}
                            }
                        }
                    }
                    c if plural => m.out.push(c),
                    _ => {}
                }
            }
            _ => m.out.push(ty),
        }
    }
}
