//! Run a BCPL source through the whole pipeline and print the verdict.
//!
//! ```text
//! cargo run --release -p phiclm --example run -- file.b [--steps 1000000] [--rows]
//! ```

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let steps = a.iter().position(|x| x == "--steps").and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let Some(path) = a.first().filter(|p| !p.starts_with("--")) else {
        eprintln!("which .b file?");
        return;
    };
    let src = std::fs::read_to_string(path).unwrap_or_default();
    let v = phiclm::judge(&src, steps);
    print!("{}", v.out);
    match &v.error {
        None => println!("— compiled, ran {} steps, returned {}.", v.steps, v.result),
        Some(e) if v.compiled => println!("— compiled, then: {e}"),
        Some(e) => println!("— did not compile: {e}"),
    }
    if a.iter().any(|x| x == "--rows") {
        if let Ok(p) = phiclm::compile(&src) {
            for r in p.rows() {
                println!("{}\t{}\t{}", r.about, r.kind, r.other);
            }
        }
    }
}
