//! BLIB's streams (`blib.b:493–548`) over the judge's own world: stream 1
//! is its input string, stream 2 its output buffer, and `findinput` opens
//! the named texts the judge was handed. The machine reads and writes only
//! the CURRENT streams (`input`/`read`, `out`); selecting another parks the
//! current one in its slot, so no writer has to know which stream it is.

use super::write::text;
use crate::machine::{Machine, Stream};

/// `findinput(name)`: a stream over the named text, or 0 when there is none.
pub(super) fn findinput(m: &mut Machine, name: i64) -> i64 {
    let name = text(m, name);
    match m.files.iter().find(|(n, _)| *n == name).map(|(_, t)| t.clone()) {
        Some(chars) => {
            m.streams.push(Stream { name, chars, read: 0, text: String::new(), output: false, open: true });
            m.streams.len() as i64
        }
        None => 0,
    }
}

/// `findoutput(name)`: a fresh stream; what is written to it is in the
/// verdict under its name.
pub(super) fn findoutput(m: &mut Machine, name: i64) -> i64 {
    let name = text(m, name);
    m.streams.push(Stream { name, chars: Vec::new(), read: 0, text: String::new(), output: true, open: true });
    m.streams.len() as i64
}

/// The open stream `id` of the wanted direction; 0 means the judge's own.
fn open(m: &Machine, id: i64, output: bool) -> Result<usize, String> {
    let id = match (id, output) {
        (0, false) => 1,
        (0, true) => 2,
        (k, _) => k as usize,
    };
    match m.streams.get(id.wrapping_sub(1)) {
        Some(s) if s.output == output && s.open => Ok(id),
        Some(_) => Err(format!("stream {id} is closed, or the other direction")),
        None => Err(format!("no stream {id}")),
    }
}

/// `selectinput(scb)`: the current input is parked, `scb` becomes current.
pub(super) fn selectinput(m: &mut Machine, id: i64) -> Result<(), String> {
    let id = open(m, id, false)?;
    let cis = m.cis;
    m.streams[cis - 1].chars = std::mem::take(&mut m.input);
    m.streams[cis - 1].read = m.read;
    m.input = std::mem::take(&mut m.streams[id - 1].chars);
    m.read = m.streams[id - 1].read;
    m.cis = id;
    Ok(())
}

/// `selectoutput(scb)`: the current output is parked, `scb` becomes current.
pub(super) fn selectoutput(m: &mut Machine, id: i64) -> Result<(), String> {
    let id = open(m, id, true)?;
    let cos = m.cos;
    m.streams[cos - 1].text = std::mem::take(&mut m.out);
    m.out = std::mem::take(&mut m.streams[id - 1].text);
    m.cos = id;
    Ok(())
}

/// `endread()`: the current input closes and the judge's own is current.
pub(super) fn endread(m: &mut Machine) -> Result<(), String> {
    let cis = m.cis;
    selectinput(m, 1)?;
    if cis != 1 {
        m.streams[cis - 1].open = false;
    }
    Ok(())
}

/// `endwrite()`: the current output closes (what it holds is kept for the
/// verdict) and the judge's own is current.
pub(super) fn endwrite(m: &mut Machine) -> Result<(), String> {
    let cos = m.cos;
    selectoutput(m, 2)?;
    if cos != 2 {
        m.streams[cos - 1].open = false;
    }
    Ok(())
}
