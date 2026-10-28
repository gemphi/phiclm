use super::host::Host;
use super::Value;
use std::collections::HashMap;

/// Lexical frames, innermost last.
pub type Env = Vec<HashMap<String, Value>>;

/// The machine: a global frame and the host that adds its primitives.
pub struct Machine<H: Host> {
    pub global: HashMap<String, Value>,
    pub host: H,
}

impl<H: Host + Default> Default for Machine<H> {
    fn default() -> Self {
        Self::with_host(H::default())
    }
}

impl<H: Host + Default> Machine<H> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<H: Host> Machine<H> {
    /// The core primitives, then the host's. A host primitive that repeats a
    /// core name is refused: a host that shadows `union` is a bug, not a choice.
    pub fn with_host(host: H) -> Self {
        let mut global = HashMap::new();
        for p in super::prims::NAMES {
            global.insert(p.to_string(), Value::Prim(p));
        }
        for p in host.names() {
            assert!(!global.contains_key(*p), "host primitive {p} repeats a core primitive");
            global.insert(p.to_string(), Value::Prim(p));
        }
        Self { global, host }
    }

    /// Read and evaluate every form in `src`; the last value.
    pub fn run(&mut self, src: &str) -> Result<Value, String> {
        let mut last = Value::Nil;
        for form in super::read(src)? {
            last = self.eval(&form, &mut Vec::new())?;
        }
        Ok(last)
    }

    /// A bound name's value; an unbound symbol is a word and stands for
    /// itself, so `(held russell is-a)` reads as a row does.
    fn lookup(&self, env: &Env, name: &str) -> Result<Value, String> {
        Ok(env.iter().rev().find_map(|f| f.get(name)).or_else(|| self.global.get(name)).cloned().unwrap_or_else(|| Value::Sym(name.to_string())))
    }

    pub fn eval(&mut self, v: &Value, env: &mut Env) -> Result<Value, String> {
        match v {
            Value::Sym(s) => self.lookup(env, s),
            Value::List(items) => {
                let head = &items[0];
                if let Value::Sym(s) = head {
                    if let Some(r) = self.special(s, &items[1..], env) {
                        return r;
                    }
                }
                let f = self.eval(head, env)?;
                let mut args = Vec::with_capacity(items.len() - 1);
                for a in &items[1..] {
                    args.push(self.eval(a, env)?);
                }
                self.apply(f, args)
            }
            other => Ok(other.clone()),
        }
    }

    /// The special forms: quote, if, define, lambda, let, begin, and, or.
    fn special(&mut self, s: &str, rest: &[Value], env: &mut Env) -> Option<Result<Value, String>> {
        Some(match s {
            "quote" => Ok(rest.first().cloned().unwrap_or(Value::Nil)),
            "if" => match self.eval(rest.first()?, env) {
                Ok(c) if c.truthy() => rest.get(1).map(|t| self.eval(t, env)).unwrap_or(Ok(Value::Nil)),
                Ok(_) => rest.get(2).map(|t| self.eval(t, env)).unwrap_or(Ok(Value::Nil)),
                Err(e) => Err(e),
            },
            "define" => {
                let name = rest.first()?.text()?;
                let val = match self.eval(rest.get(1)?, env) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                match env.last_mut() {
                    Some(frame) => frame.insert(name.clone(), val),
                    None => self.global.insert(name.clone(), val),
                };
                Ok(Value::Sym(name))
            }
            "lambda" => {
                let params = match rest.first()? {
                    Value::List(ps) => ps.iter().filter_map(Value::text).collect(),
                    Value::Nil => Vec::new(),
                    _ => return Some(Err("lambda: (params) expected".into())),
                };
                Ok(Value::Lambda(params, Box::new(Value::List(std::iter::once(Value::Sym("begin".into())).chain(rest[1..].iter().cloned()).collect())), env.clone()))
            }
            "let" => {
                let mut frame = HashMap::new();
                if let Value::List(binds) = rest.first()? {
                    for b in binds {
                        if let Value::List(kv) = b {
                            let val = match self.eval(kv.get(1)?, env) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(e)),
                            };
                            frame.insert(kv.first()?.text()?, val);
                        }
                    }
                }
                env.push(frame);
                let r = self.begin(&rest[1..], env);
                env.pop();
                r
            }
            "begin" => self.begin(rest, env),
            // Data constructors read their arguments as written: `(pairs (a b) (b c))`.
            "pairs" | "class" => super::prims::call(self, if s == "pairs" { "pairs" } else { "class" }, rest.to_vec()),
            "and" => {
                for r in rest {
                    match self.eval(r, env) {
                        Ok(v) if !v.truthy() => return Some(Ok(v)),
                        Err(e) => return Some(Err(e)),
                        _ => {}
                    }
                }
                Ok(Value::Bool(true))
            }
            "or" => {
                for r in rest {
                    match self.eval(r, env) {
                        Ok(v) if v.truthy() => return Some(Ok(v)),
                        Err(e) => return Some(Err(e)),
                        _ => {}
                    }
                }
                Ok(Value::Bool(false))
            }
            _ => return None,
        })
    }

    fn begin(&mut self, forms: &[Value], env: &mut Env) -> Result<Value, String> {
        let mut last = Value::Nil;
        for f in forms {
            last = self.eval(f, env)?;
        }
        Ok(last)
    }

    pub fn apply(&mut self, f: Value, args: Vec<Value>) -> Result<Value, String> {
        match f {
            Value::Prim(name) => super::prims::call(self, name, args),
            Value::Lambda(params, body, closed) => {
                let mut env = closed;
                env.push(params.iter().cloned().zip(args).collect());
                self.eval(&body, &mut env)
            }
            other => Err(format!("not a function: {other}")),
        }
    }
}
