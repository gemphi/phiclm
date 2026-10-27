# phiclm — architecture

## The crate, folder by folder

```mermaid
graph TD
    lib["lib.rs"]
    subgraph front["front end — BCPL's LEX · SYN"]
        token["token/<br/>mod · number · text · skip · section · tests"]
        parse["parse/<br/>mod · expression · command"]
        tree["tree.rs"]
    end
    subgraph middle["TRN — compression"]
        trn["trn/<br/>mod · routine · order · express · declare<br/>loops · switch · field · fold · labels"]
        ocode["ocode.rs<br/>55 ops"]
    end
    subgraph back["the machine — cinterp's place"]
        machine["machine/<br/>mod · run · call · co · context · arith · heap"]
        global["global/<br/>mod · slots · format · write · read · stream"]
    end
    subgraph out["the judge"]
        judge["judge.rs"]
        tell["tell.rs"]
    end
    manifest["manifest/<br/>mod · ops · tokens · nodes · tests"]
    tests["tests/<br/>core · control · data · syntax · blib · threads · streams"]

    lib --> token & parse & tree & trn & ocode & machine & global & judge & tell & manifest
    token --> parse --> tree --> trn --> ocode --> machine
    global --- machine
    machine --> judge --> tell
    manifest -.kind.-> token & tree & ocode
    tests -.judge.-> judge
```

## The pipeline

```mermaid
graph LR
    SRC["BCPL source"] --> LEX["token/ · lex"] --> SYN["parse/ · rexp(n)"] --> TREE["tree"] --> TRN["trn/ · translate"] --> OC["O-code"] --> RUN["machine/ · run"]
    BLIB["global/ · BLIB"] --- RUN
    IN["input string"] -.rdch.-> RUN
    RUN --> V["Verdict<br/>compiled · error · out · result · steps"] --> ROWS["tell · Rows"] --> W["phiecs World · phiano student"]
```

## The module diagrams

```mermaid
graph LR
    A["ARCHITECTURE.md"] --> L["token.md<br/>token/"]
    A --> P["parse.md<br/>parse/ · tree.rs"]
    A --> T["trn.md<br/>trn/"]
    A --> M["machine.md<br/>machine/ · ocode.rs"]
    A --> B["global.md<br/>global/"]
    A --> D["manifest.md<br/>manifest/"]
    A --> J["judge.md<br/>judge.rs · tell.rs · tests/"]
    A --> C["bcpl.md<br/>BCPL, file for file"]
    A --> R["prose.md<br/>the corpus · two compilers"]
```

| diagram | file |
|---|---|
| the lexer | [docs/phiclm/token.md](../../docs/phiclm/token.md) |
| the parser and the tree | [docs/phiclm/parse.md](../../docs/phiclm/parse.md) |
| the translator | [docs/phiclm/trn.md](../../docs/phiclm/trn.md) |
| the machine and O-code | [docs/phiclm/machine.md](../../docs/phiclm/machine.md) |
| BLIB | [docs/phiclm/global.md](../../docs/phiclm/global.md) |
| the ladder | [docs/phiclm/manifest.md](../../docs/phiclm/manifest.md) |
| the judge | [docs/phiclm/judge.md](../../docs/phiclm/judge.md) |
| BCPL, file for file | [docs/phiclm/bcpl.md](../../docs/phiclm/bcpl.md) |
| prose and the two compilers | [docs/phiclm/prose.md](../../docs/phiclm/prose.md) |

## Status

2026-10-27 · 38 tests + 1 doc test · every .rs file ≤ 200 lines · what is left: [TODO.md](TODO.md).
