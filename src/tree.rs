//! The tree `trn` walks: one expression, one command, one declaration —
//! operators, not a class per node. BCPL's whole grammar fits in three
//! enums because the language has one type, the word.

/// An expression: everything that has a value.
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Num(i64),
    Name(String),
    Str(String),
    True,
    False,
    /// `-e  !e  @e  ~e  abs e`
    Monad(&'static str, Box<Expression>),
    /// `a ! b`, `a + b`, `a = b`, … spelled as the op's canonical symbol.
    Dyad(&'static str, Box<Expression>, Box<Expression>),
    /// CPL's gift: `e -> a, b`.
    Cond(Box<Expression>, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Valof(Box<Command>),
    /// `VEC k` — k words of frame, its value their address.
    Vec(i64),
}

/// A command: everything done for effect.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// `$( … $)` — declarations and commands in written order.
    Block(Vec<Element>),
    Assign(Expression, Expression),
    /// A routine call for its effect.
    Do(Expression),
    If(Expression, Box<Command>),
    Unless(Expression, Box<Command>),
    Test(Expression, Box<Command>, Box<Command>),
    While(Expression, Box<Command>),
    Until(Expression, Box<Command>),
    Repeat(Box<Command>),
    RepeatWhile(Box<Command>, Expression),
    RepeatUntil(Box<Command>, Expression),
    For(String, Expression, Expression, Option<Expression>, Box<Command>),
    Resultis(Expression),
    Return,
    Finish,
    Break,
    Loop,
}

/// One slot of a block, in written order: a run of `LET … AND …`, or a
/// command. Scope runs from the declaration to the block's end, as in BCPL.
#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Let(Vec<Declaration>),
    Do(Command),
}

/// What a `LET` introduces.
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    /// `LET f(a, b) = e` — a function.
    Function(String, Vec<String>, Expression),
    /// `LET r(a) BE c` — a routine.
    Routine(String, Vec<String>, Command),
    /// `LET x = e` — a variable (in a frame, or a global at top level).
    Variable(String, Expression),
    /// `MANIFEST $( name = k; … $)` — named constants.
    Manifest(Vec<(String, i64)>),
    /// `GLOBAL $( name: slot; … $)` — named global-vector slots.
    Global(Vec<(String, usize)>),
}
