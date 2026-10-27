//! The machine's arithmetic on words: integer, logical and floating, one
//! function for the monads and one for the dyads.

use crate::ocode::Op;

/// One monadic operation on a word, as the machine does it.
pub(crate) fn unary(op: Op, a: i64) -> i64 {
    let f = f64::from_bits(a as u64);
    match op {
        Op::Neg => a.wrapping_neg(),
        Op::Not => !a,
        Op::Abs => a.wrapping_abs(),
        Op::Float => (a as f64).to_bits() as i64,
        Op::Fix => f.round() as i64,
        Op::Fneg => (-f).to_bits() as i64,
        Op::Fabs => f.abs().to_bits() as i64,
        _ => a,
    }
}

/// One dyadic operation on two words, as the machine does it — `trn`
/// folds constants with this too, so a constant means what it computes.
pub(crate) fn binary(op: Op, a: i64, b: i64) -> Result<i64, String> {
    let truth = |t: bool| match t {
        true => -1,
        false => 0,
    };
    let (x, y) = (f64::from_bits(a as u64), f64::from_bits(b as u64));
    let float = |v: f64| v.to_bits() as i64;
    Ok(match op {
        Op::Add => a.wrapping_add(b),
        Op::Sub => a.wrapping_sub(b),
        Op::Mul => a.wrapping_mul(b),
        Op::Div => a.checked_div(b).ok_or("division by zero")?,
        Op::Rem => a.checked_rem(b).ok_or("REM by zero")?,
        Op::And => a & b,
        Op::Or => a | b,
        Op::Lsh => match b {
            0..=63 => a.wrapping_shl(b as u32),
            _ => 0,
        },
        Op::Rsh => match b {
            0..=63 => ((a as u64) >> b) as i64,
            _ => 0,
        },
        Op::Eqv => !(a ^ b),
        Op::Neqv => a ^ b,
        Op::Eq => truth(a == b),
        Op::Ne => truth(a != b),
        Op::Ls => truth(a < b),
        Op::Gr => truth(a > b),
        Op::Le => truth(a <= b),
        Op::Ge => truth(a >= b),
        Op::Fadd => float(x + y),
        Op::Fsub => float(x - y),
        Op::Fmul => float(x * y),
        Op::Fdiv => float(x / y),
        Op::Feq => truth(x == y),
        Op::Fne => truth(x != y),
        Op::Fls => truth(x < y),
        Op::Fgr => truth(x > y),
        Op::Fle => truth(x <= y),
        Op::Fge => truth(x >= y),
        other => return Err(format!("{} is not dyadic", other.name())),
    })
}
