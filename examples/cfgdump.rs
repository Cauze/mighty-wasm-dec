//! Debug helper: dump the CFG of a lifted function.
//! `cargo run --example cfgdump <file> [--func-name N]`

use mighty_wasm_dec::{cfg, lift, parse};

fn main() -> anyhow::Result<()> {
    // Deep functions (f2422: ~990 nesting levels) overflow the default
    // 8MB stack in the recursive CFG builder — same big-stack treatment
    // as the main binary.
    std::thread::Builder::new()
        .name("cfgdump".into())
        .stack_size(512 * 1024 * 1024)
        .spawn(run)
        .map_err(|e| anyhow::anyhow!("spawn worker: {e}"))?
        .join()
        .unwrap_or_else(|_| Err(anyhow::anyhow!("worker panicked")))
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: cfgdump <file> [func-name]");
    let name = args.get(2).cloned();
    let bytes = if std::path::Path::new(path).extension().and_then(|e| e.to_str()) == Some("wat") {
        wat::parse_str(&std::fs::read_to_string(path)?)?
    } else {
        std::fs::read(path)?
    };
    let mut meta = parse::parse_meta(&bytes)?;
    let mir = lift::lift_module(&bytes, &mut meta)?;
    for f in &mir.funcs {
        if let Some(n) = &name {
            if &f.name != n {
                continue;
            }
        }
        let g = cfg::build(&f.body);
        println!("== {} ({} blocks, entry {})", f.name, g.blocks.len(), g.entry);
        for b in &g.blocks {
            let term = match &b.term {
                cfg::Terminator::Next(t) => format!("next {t}"),
                cfg::Terminator::Jump(t) => format!("jump {t}"),
                cfg::Terminator::Branch { then_b, else_b, .. } => {
                    format!("br {then_b}/{else_b}")
                }
                cfg::Terminator::Switch { arms, default, .. } => {
                    format!("switch {arms:?} else {default}")
                }
                cfg::Terminator::Return(v) => format!("return({})", v.len()),
                cfg::Terminator::Unreachable => "unreachable".into(),
            };
            println!(
                "  b{} [{}] preds={:?} stmts={} term={term}",
                b.id,
                b.label,
                b.preds,
                b.stmts.len()
            );
            for s in &b.stmts {
                println!("    {s:?}");
            }
        }
    }
    Ok(())
}
