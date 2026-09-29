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
