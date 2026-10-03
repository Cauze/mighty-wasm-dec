use std::collections::HashMap;
use std::fmt;

#[allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmTy {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl WasmTy {
    pub fn to_c(&self) -> &'static str {
        match self {
            WasmTy::I32 => "int32_t",
            WasmTy::I64 => "int64_t",
            WasmTy::F32 => "float",
            WasmTy::F64 => "double",
            WasmTy::V128 => "__v128",
            WasmTy::FuncRef => "void*",
            WasmTy::ExternRef => "void*",
        }
    }
}

impl fmt::Display for WasmTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_c())
    }
}

/// A named value slot: wasm local, synthetic tmp, or global.
/// Typed replacement for `"l{i}"`/`"t{i}"`/`"g{i}"` strings in pass maps —
/// integer comparison instead of `format!` + string compare per node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Var {
    Local(u32),
    Tmp(u32),
    Global(u32),
}

impl Var {
    /// Rendered name (`l3`, `t0`, `g1`); matches the historical strings so
    /// sort order and diagnostics are unchanged.
    pub fn name(&self) -> String {
        match self {
            Var::Local(i) => format!("l{i}"),
            Var::Tmp(i) => format!("t{i}"),
            Var::Global(i) => format!("g{i}"),
        }
    }

    /// Parse a `dst`-style name; `None` for struct keys etc.
    pub fn parse(s: &str) -> Option<Var> {
        if s.len() < 2 || !s.is_ascii() {
            return None;
        }
        let (k, rest) = (&s[..1], &s[1..]);
        let i: u32 = rest.parse().ok()?;
        match k {
            "l" => Some(Var::Local(i)),
            "t" => Some(Var::Tmp(i)),
            "g" => Some(Var::Global(i)),
            _ => None,
        }
    }

    pub fn is_tmp(&self) -> bool {
        matches!(self, Var::Tmp(_))
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    ConstI32(i32),
    ConstI64(i64),
    ConstF32(u32), // bits
    ConstF64(u64), // bits
    Local(u32),
    Tmp(u32),
    Global(u32),
    MemorySize(u32),
    Binop {
        op: String,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unop {
        op: String,
        v: Box<Expr>,
    },
    Load {
        ty: String,
        base: Box<Expr>,
        offset: u64,
    },
    Call {
        func: u32,
        name: String,
        args: Vec<Expr>,
    },
    CallIndirect {
        type_idx: u32,
        table: u32,
        index: Box<Expr>,
        args: Vec<Expr>,
    },
    Select {
        c: Box<Expr>,
        a: Box<Expr>,
        b: Box<Expr>,
    },
    /// Generic model for SIMD/atomic/GC/ref/bulk ops: `op(a, b, ...)`.
    /// Keeps exact stack arity with honest C-like rendering.
    Simd {
        op: String,
        args: Vec<Expr>,
    },
    Raw(String),
    Unknown(String),
}

impl Expr {
    /// True if evaluating this has no observable effect (safe to
    /// duplicate or eliminate).
    pub fn is_pure(&self) -> bool {
        match self {
            Expr::ConstI32(_)
            | Expr::ConstI64(_)
            | Expr::ConstF32(_)
            | Expr::ConstF64(_)
            | Expr::Local(_)
            | Expr::Tmp(_)
            | Expr::Global(_) => true,
            Expr::Binop { lhs, rhs, .. } => lhs.is_pure() && rhs.is_pure(),
            Expr::Unop { v, .. } => v.is_pure(),
            Expr::Select { c, a, b } => c.is_pure() && a.is_pure() && b.is_pure(),
            _ => false,
        }
    }

    /// Expression-tree depth (inlining budget heuristic).
    pub fn depth(&self) -> usize {
        match self {
            Expr::Binop { lhs, rhs, .. } => 1 + lhs.depth().max(rhs.depth()),
            Expr::Unop { v, .. } => 1 + v.depth(),
            Expr::Select { c, a, b } => 1 + c.depth().max(a.depth().max(b.depth())),
            Expr::Load { base, .. } => 1 + base.depth(),
            Expr::Call { args, .. } => 1 + args.iter().map(|a| a.depth()).max().unwrap_or(0),
            Expr::CallIndirect { index, args, .. } => {
                1 + index.depth().max(args.iter().map(|a| a.depth()).max().unwrap_or(0))
            }
            _ => 0,
        }
    }

    /// Variable slots mentioned in this expression.
    pub fn vars(&self, acc: &mut Vec<Var>) {
        match self {
            Expr::Local(i) => acc.push(Var::Local(*i)),
            Expr::Tmp(i) => acc.push(Var::Tmp(*i)),
            Expr::Global(i) => acc.push(Var::Global(*i)),
            Expr::Binop { lhs, rhs, .. } => {
                lhs.vars(acc);
                rhs.vars(acc);
            }
            Expr::Unop { v, .. } => v.vars(acc),
            Expr::Load { base, .. } => base.vars(acc),
            Expr::Call { args, .. } => {
                for a in args {
                    a.vars(acc);
                }
            }
            Expr::CallIndirect { index, args, .. } => {
                index.vars(acc);
                for a in args {
                    a.vars(acc);
                }
            }
            Expr::Select { c, a, b } => {
                c.vars(acc);
                a.vars(acc);
                b.vars(acc);
            }
            Expr::Simd { args, .. } => {
                for a in args {
                    a.vars(acc);
                }
            }
            _ => {}
        }
    }

    /// Strip conversion-cast wrappers (`(uint32_t)`, `(int64_t)`, …) to the
    /// core expression. Casts are pure, so stripping for *recognition* is
    /// sound (rewrites re-cast as needed).
    pub fn strip_casts(&self) -> &Expr {
        let mut cur = self;
        loop {
            match cur {
                Expr::Unop { op, v }
                    if (op.starts_with("(u") || op.starts_with("(i"))
                        && op.ends_with(')') =>
                {
                    cur = v;
                }
                _ => return cur,
            }
        }
    }

    pub fn render(&self) -> String {
        match self {
            Expr::ConstI32(v) => format!("{v}"),
            Expr::ConstI64(v) => format!("{v}ll"),
            Expr::ConstF32(b) => format!("f32({:#x})", b),
            Expr::ConstF64(b) => format!("f64({:#x})", b),
            Expr::Local(i) => format!("l{i}"),
            Expr::Tmp(i) => format!("t{i}"),
            Expr::Global(i) => format!("g{i}"),
            Expr::MemorySize(m) => format!("memory_size({m})"),
            Expr::Binop { op, lhs, rhs } => format!("({} {} {})", lhs.render(), op, rhs.render()),
            Expr::Unop { op, v } => format!("({}{})", op, v.render()),
            Expr::Load { ty, base, offset } => {
                if *offset == 0 {
                    format!("LOAD_{ty}({})", base.render())
                } else {
                    format!("LOAD_{ty}({}+{offset})", base.render())
                }
            }
            Expr::Call { func: _, name, args } => {
                let a: Vec<String> = args.iter().map(|e| e.render()).collect();
                format!("{name}({})", a.join(", "))
            }
            Expr::CallIndirect { index, args, .. } => {
                let a: Vec<String> = args.iter().map(|e| e.render()).collect();
                format!("table_call({})({})", index.render(), a.join(", "))
            }
            Expr::Select { c, a, b } => {
                format!("({} ? {} : {})", c.render(), a.render(), b.render())
            }
            Expr::Simd { op, args } => {
                let a: Vec<String> = args.iter().map(|e| e.render()).collect();
                format!("{op}({})", a.join(", "))
            }
            Expr::Raw(s) => s.clone(),
            Expr::Unknown(s) => format!("/*{s}*/0"),
        }
    }

    /// Best-effort const i32 value (for call_indirect resolution).
    pub fn const_i32(&self) -> Option<i32> {
        if let Expr::ConstI32(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign { dst: String, expr: Expr },
    Store { ty: String, base: Expr, offset: u64, value: Expr },
    ExprStmt(Expr),
    If { cond: Expr, then_b: Vec<Stmt>, else_b: Vec<Stmt> },
    Block { label: String, body: Vec<Stmt> },
    Loop { label: String, body: Vec<Stmt> },
    Br { depth: u32, label: String, is_loop: bool },
    BrIf { depth: u32, label: String, is_loop: bool, cond: Expr },
    BrTable { index: Expr, targets: Vec<(u32, String, bool)>, default: (u32, String, bool) },
    Return { values: Vec<Expr> },
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct MemAccess {
    pub base_desc: String,
    pub offset: u64,
    pub width: u8,
    pub is_write: bool,
    pub dom_ty: String,
}

#[derive(Debug, Clone)]
pub struct FuncIR {
    pub idx: u32,
    pub name: String,
    pub params: Vec<WasmTy>,
    pub results: Vec<WasmTy>,
    pub locals: Vec<WasmTy>,
    pub body: Vec<Stmt>,
    pub accesses: Vec<MemAccess>,
    pub calls: Vec<u32>,
    pub indirects: Vec<Option<u32>>,
    /// Copy-propagation aliases (old base -> new base) for struct-layout keys.
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleIR {
    pub funcs: Vec<FuncIR>,
    /// table_index -> slot -> func idx (None = empty)
    pub tables: Vec<Vec<Option<u32>>>,
    pub memories: u32,
    pub data: Vec<DataSeg>,
    pub globals: Vec<GlobalDesc>,
    pub imports: Vec<ImportDesc>,
}

#[derive(Debug, Clone)]
pub struct DataSeg {
    pub idx: u32,
    pub offset: Option<i32>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GlobalDesc {
    pub idx: u32,
    pub ty: WasmTy,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct ImportDesc {
    pub module: String,
    pub name: String,
    pub kind: String,
    pub params: Vec<WasmTy>,
    pub results: Vec<WasmTy>,
}
