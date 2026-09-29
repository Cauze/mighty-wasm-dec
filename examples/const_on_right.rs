//! Example external pass: canonicalize commutative binops to const-on-right.
//!
//! `(5 + l0)` becomes `(l0 + 5)`. This helps downstream pattern matching
//! (address arithmetic, string-table indexing) by giving equivalent
//! expressions one canonical shape.
//!
//! Uses ONLY the public library API (`mighty_wasm_dec::...`) — the same
//! boundary a third-party deobfuscation pass would use.
//!
//! Run: `cargo run --example const_on_right corpus/wat/mvp-01-fac.wat`

use mighty_wasm_dec::{emit, ir, lift, parse, passes};
use passes::{FuncPass, PassCtx};

/// Put constants on the right of commutative binops.
struct CommutativeCanon;

fn is_const(e: &ir::Expr) -> bool {
    matches!(
        e,
        ir::Expr::ConstI32(_)
            | ir::Expr::ConstI64(_)
            | ir::Expr::ConstF32(_)
            | ir::Expr::ConstF64(_)
    )
}

fn is_commutative(op: &str) -> bool {
    matches!(op, "+" | "*" | "&" | "|" | "^" | "==" | "!=")
}

fn canon_expr(e: &mut ir::Expr) -> bool {
    let mut changed = false;
    match e {
        ir::Expr::Binop { op, lhs, rhs } => {
            changed |= canon_expr(lhs);
            changed |= canon_expr(rhs);
            if is_commutative(op) && is_const(lhs) && !is_const(rhs) {
                std::mem::swap(lhs, rhs);
                changed = true;
            }
        }
        ir::Expr::Unop { v, .. } => changed |= canon_expr(v),
        ir::Expr::Load { base, .. } => changed |= canon_expr(base),
        ir::Expr::Call { args, .. } => {
            for a in args {
                changed |= canon_expr(a);
            }
        }
        ir::Expr::CallIndirect { index, args, .. } => {
            changed |= canon_expr(index);
            for a in args {
                changed |= canon_expr(a);
            }
        }
        ir::Expr::Select { c, a, b } => {
            changed |= canon_expr(c);
            changed |= canon_expr(a);
            changed |= canon_expr(b);
        }
        ir::Expr::Simd { args, .. } => {
            for a in args {
                changed |= canon_expr(a);
            }
        }
        _ => {}
    }
    changed
}

fn canon_stmts(stmts: &mut [ir::Stmt]) -> bool {
    let mut changed = false;
    for s in stmts.iter_mut() {
        match s {
            ir::Stmt::Assign { expr, .. } => changed |= canon_expr(expr),
            ir::Stmt::Store { base, value, .. } => {
                changed |= canon_expr(base);
                changed |= canon_expr(value);
            }
            ir::Stmt::ExprStmt(e) => changed |= canon_expr(e),
            ir::Stmt::If { cond, then_b, else_b } => {
                changed |= canon_expr(cond);
                changed |= canon_stmts(then_b);
                changed |= canon_stmts(else_b);
            }
            ir::Stmt::Block { body, .. } | ir::Stmt::Loop { body, .. } => {
                changed |= canon_stmts(body)
            }
            ir::Stmt::BrIf { cond, .. } => changed |= canon_expr(cond),
            ir::Stmt::BrTable { index, .. } => changed |= canon_expr(index),
            ir::Stmt::Return { values } => {
                for v in values {
                    changed |= canon_expr(v);
                }
            }
            _ => {}
        }
    }
    changed
}

impl FuncPass for CommutativeCanon {
    fn name(&self) -> &'static str {
        "commutative-canon"
    }
    fn run(&self, f: &mut ir::FuncIR, _ctx: &mut PassCtx) -> bool {
        canon_stmts(&mut f.body)
    }
}

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).expect("usage: const_on_right <file.wat|file.wasm>");
    let bytes = if std::path::Path::new(&path).extension().and_then(|e| e.to_str()) == Some("wat") {
        wat::parse_str(&std::fs::read_to_string(&path)?)?
    } else {
        std::fs::read(&path)?
    };
    let mut meta = parse::parse_meta(&bytes)?;
    let mut mir = lift::lift_module(&bytes, &mut meta)?;
    let pass = CommutativeCanon;
    let mut ctx = PassCtx::default();
    for f in mir.funcs.iter_mut() {
        passes::optimize_func(f);
        for _ in 0..4 {
            if !pass.run(f, &mut ctx) {
                break;
            }
        }
    }
    print!("{}", emit::emit_c(&mir));
    Ok(())
}
