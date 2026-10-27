//! A thread of control — the main program or one coroutine — and the
//! coroutines the machine holds: BLIB's `co_*` fields (`blib.b:1262`) as a
//! Rust struct, with the stack each one runs on inside its `getvec` block.

/// Where a thread is: its frame pointer, its next op, its expression stack,
/// and each live frame's P with the stack depth it was entered at (so a
/// `longjump` can cut back to it), innermost last.
#[derive(Debug, Default)]
pub(crate) struct Context {
    pub p: i64,
    pub pc: usize,
    pub stack: Vec<i64>,
    pub frames: Vec<(i64, usize)>,
    /// A coroutine's stack ends with its block; the main program runs up
    /// to the heap instead.
    pub end: Option<i64>,
}

impl Context {
    pub fn pop(&mut self) -> Result<i64, String> {
        self.stack.pop().ok_or_else(|| "stack underflow".to_string())
    }
}

/// One coroutine: its pointer (the address of its stack block, as BLIB
/// hands out), the coroutine that called it (0 while it waits for no one),
/// its body and stack size, and where it stopped. A coroutine with no saved
/// context is fresh: the next transfer to it runs `body(value)`.
#[derive(Debug)]
pub(crate) struct Co {
    pub cptr: i64,
    pub parent: i64,
    pub body: i64,
    pub size: i64,
    pub saved: Option<Context>,
}

/// What a step leaves the run loop to do.
pub(crate) enum Flow {
    Go,
    Done(i64),
}
