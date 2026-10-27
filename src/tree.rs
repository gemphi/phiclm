//! The tree `trn` walks: one expression, one command, one declaration —
//! operators, not a class per node. BCPL's whole grammar fits in three
//! enums because the language has one type, the word.

/// An expression: everything that has a value.
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Num(i64),
    /// A floating constant; the word holds its bits.
    Fnum(f64),
    Name(String),
    Str(String),
    True,
    False,
    /// `-e  !e  @e  ~e  abs e`, and the floating `#-e  #abs e  float e  fix e`.
    Monad(&'static str, Box<Expression>),
    /// `a ! b`, `a + b`, `a = b`, `f of p`, `a #+ b`, … spelled as the op's
    /// canonical symbol.
    Dyad(&'static str, Box<Expression>, Box<Expression>),
    /// CPL's gift: `e -> a, b`.
    Cond(Box<Expression>, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Valof(Box<Command>),
    /// `VEC k` — k words of frame, its value their address; `k` folds.
    Vec(Box<Expression>),
    /// `TABLE k1, k2, …` — a constant vector in the store, its value the
    /// address of its first word (`bcplfe.b:1809`).
    Table(Vec<Expression>),
    /// `SLCT len:sh:offset` — a field selector, folded to the one word
    /// `len<<24 | sh<<16 | offset` that `OF` reads (`bcplfe.b:3361`).
    Slct(Box<Expression>, Box<Expression>, Box<Expression>),
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
    /// `SWITCHON e INTO $( … $)` — the body holds the `CASE` labels.
    Switchon(Expression, Box<Command>),
    /// `CASE k: c` — a label inside a SWITCHON; `k` folds to a constant.
    Case(Expression, Box<Command>),
    /// `DEFAULT: c`.
    Default(Box<Command>),
    /// Out of the innermost SWITCHON.
    Endcase,
    /// `GOTO e` — to a label of the same routine; `e` is usually its name,
    /// and may be any expression whose value is one (`bcplfe.b:2574`).
    Goto(Expression),
    /// `L: c` — a label, then the command it names (`bcplfe.b:1984`).
    Label(String, Box<Command>),
}

/// One slot of a block, in written order: a run of `LET … AND …`, or a
/// command. Scope runs from the declaration to the block's end, as in BCPL.
#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Let(Vec<Declaration>),
    Do(Command),
}

/// A declaration block's rows: a name and its constant expression, or no
/// expression for the previous value plus one, counting from -1
/// (`bcplfe.b:2524`).
pub type Rows = Vec<(String, Option<Expression>)>;

/// What a `LET` introduces.
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    /// `LET f(a, b) = e` — a function.
    Function(String, Vec<String>, Expression),
    /// `LET r(a) BE c` — a routine.
    Routine(String, Vec<String>, Command),
    /// `LET x = e` — a variable (in a frame, or a constant at top level);
    /// `LET a, b = x, y` is two of them, declared together.
    Variable(String, Expression),
    /// `MANIFEST $( name = k; … $)` — named constants.
    Manifest(Rows),
    /// `GLOBAL $( name: slot; … $)` — named global-vector slots.
    Global(Rows),
    /// `STATIC $( name = k; … $)` — named cells of the store, each with its
    /// initial value: they keep their value between calls.
    Static(Rows),
}
