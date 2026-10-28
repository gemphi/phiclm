//! The host: what a world adds to the machine — its own primitives, bound
//! after the core's and called by name with their arguments evaluated. `()`
//! is the host of the bare machine.

use super::Value;

pub trait Host {
    /// The primitives this host adds.
    fn names(&self) -> &'static [&'static str];
    /// One of them, called with its arguments evaluated.
    fn call(&mut self, name: &'static str, args: Vec<Value>) -> Result<Value, String>;
}

impl Host for () {
    fn names(&self) -> &'static [&'static str] {
        &[]
    }

    fn call(&mut self, name: &'static str, _: Vec<Value>) -> Result<Value, String> {
        Err(format!("unknown primitive {name}"))
    }
}
