use anyhow::{Context, Result};
use std::collections::HashMap;
use wasmparser::{CompositeInnerType, KnownCustom, Name, Payload};

use crate::ir::WasmTy;

fn wty(v: &wasmparser::ValType) -> WasmTy {
    match v {
        wasmparser::ValType::I32 => WasmTy::I32,
        wasmparser::ValType::I64 => WasmTy::I64,
        wasmparser::ValType::F32 => WasmTy::F32,
        wasmparser::ValType::F64 => WasmTy::F64,
        wasmparser::ValType::V128 => WasmTy::V128,
        wasmparser::ValType::Ref(r) => {
            if r.is_func_ref() {
                WasmTy::FuncRef
            } else {
                WasmTy::ExternRef
            }
        }
    }
}

#[derive(Default)]
pub struct ModuleMeta {
    pub types: Vec<(Vec<WasmTy>, Vec<WasmTy>)>,
    pub func_types: Vec<u32>,
    pub exports: HashMap<u32, String>,
    pub func_names: HashMap<u32, String>,
    pub tables: Vec<Vec<Option<u32>>>,
    pub memories: u32,
    pub data: Vec<(Option<i32>, Vec<u8>)>,
    pub globals: Vec<(WasmTy, bool)>,
    pub imports: Vec<(String, String, String)>,
    pub import_func_count: u32,
    pub import_func_types: Vec<u32>,
    pub import_func_names: Vec<String>,
    /// GC struct type index -> field count (for StructNew arity).
    pub struct_arity: HashMap<u32, usize>,
}

impl ModuleMeta {
    /// Signature for any function index (imports first, then defined).
    pub fn func_sig(&self, idx: u32) -> (Vec<WasmTy>, Vec<WasmTy>) {
        let ty_idx = if idx < self.import_func_count {
            self.import_func_types.get(idx as usize).cloned()
        } else {
            self.func_types
                .get((idx - self.import_func_count) as usize)
                .cloned()
        };
        ty_idx
            .and_then(|t| self.types.get(t as usize).cloned())
            .unwrap_or((Vec::new(), Vec::new()))
    }

    /// Display name for any function index; imports use `module_name`.
    pub fn call_name(&self, idx: u32) -> String {
        if idx < self.import_func_count {
            if let Some(n) = self.import_func_names.get(idx as usize) {
                return n.clone();
            }
            return format!("import{idx}");
        }
        func_display_name(idx, self)
    }
}

pub fn parse_meta(bytes: &[u8]) -> Result<ModuleMeta> {
    let mut meta = ModuleMeta::default();
    meta.tables.push(Vec::new());

    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        let payload = payload.context("parse payload")?;
        match payload {
            Payload::TypeSection(r) => {
                for group in r.into_iter() {
                    let group = group?;
                    for sub in group.into_types() {
                        let type_idx = meta.types.len() as u32;
                        match &sub.composite_type.inner {
                            CompositeInnerType::Func(ft) => {
                                let params = ft.params().iter().map(wty).collect();
                                let results = ft.results().iter().map(wty).collect();
                                meta.types.push((params, results));
                            }
                            CompositeInnerType::Struct(st) => {
                                meta.struct_arity.insert(type_idx, st.fields.len());
                                // placeholder so func-type indices stay aligned
                                meta.types.push((Vec::new(), Vec::new()));
                            }
                            _ => {
                                // array/cont/etc: placeholder so type indices stay aligned
                                meta.types.push((Vec::new(), Vec::new()));
                            }
                        }
                    }
                }
            }
            Payload::FunctionSection(r) => {
                for ty in r.into_iter() {
                    meta.func_types.push(ty?);
                }
            }
            Payload::MemorySection(r) => {
                meta.memories = r.count();
            }
            Payload::ImportSection(r) => {
                for im in r.into_iter() {
                    let im = im?;
                    let kind = match &im.ty {
                        wasmparser::TypeRef::Func(ty_idx) => {
                            meta.import_func_count += 1;
                            meta.import_func_types.push(*ty_idx);
                            meta.import_func_names
                                .push(sanitize(&format!("{}_{}", im.module, im.name)));
                            "func"
                        }
                        wasmparser::TypeRef::Memory(_) => "memory",
                        wasmparser::TypeRef::Table(_) => "table",
                        wasmparser::TypeRef::Global(_) => "global",
                        wasmparser::TypeRef::Tag(_) => "tag",
                    };
                    meta.imports
                        .push((im.module.to_string(), im.name.to_string(), kind.to_string()));
                }
            }
            Payload::GlobalSection(r) => {
                for (i, g) in r.into_iter().enumerate() {
                    let g = g?;
                    meta.globals.push((wty(&g.ty.content_type), g.ty.mutable));
                    let _ = i;
                }
            }
            Payload::DataSection(r) => {
                for (i, d) in r.into_iter().enumerate() {
                    let d = d?;
                    let off = match &d.kind {
                        wasmparser::DataKind::Active { offset_expr, .. } => {
                            let mut or = offset_expr.get_operators_reader();
                            let mut v = None;
                            while !or.eof() {
                                if let Ok(wasmparser::Operator::I32Const { value }) = or.read() {
                                    v = Some(value);
                                }
                            }
                            v
                        }
                        _ => None,
                    };
                    meta.data.push((off, d.data.to_vec()));
                    let _ = i;
                }
            }
            Payload::ExportSection(r) => {
                for e in r.into_iter() {
                    let e = e?;
                    if e.kind == wasmparser::ExternalKind::Func {
                        meta.exports.insert(e.index, e.name.to_string());
                    }
                }
            }
            Payload::ElementSection(r) => {
                for el in r.into_iter() {
                    let el = el?;
                    match el.kind {
                        wasmparser::ElementKind::Active {
                            table_index,
                            offset_expr,
                        } => {
                            let mut funcs: Vec<u32> = Vec::new();
                            match el.items {
                                wasmparser::ElementItems::Functions(reader) => {
                                    for f in reader.into_iter() {
                                        funcs.push(f?);
                                    }
                                }
                                wasmparser::ElementItems::Expressions(_, reader) => {
                                    for expr in reader.into_iter() {
                                        let expr = expr?;
                                        let mut found = None;
                                        let mut rdr = expr.get_operators_reader();
                                        while !rdr.eof() {
                                            match rdr.read()? {
                                                wasmparser::Operator::RefFunc { function_index } => {
                                                    found = Some(function_index);
                                                    break;
                                                }
                                                _ => {}
                                            }
                                        }
                                        funcs.push(found.unwrap_or(u32::MAX));
                                    }
                                }
                            }
                            let mut offset: usize = 0;
                            let mut or = offset_expr.get_operators_reader();
                            while !or.eof() {
                                if let wasmparser::Operator::I32Const { value } = or.read()? {
                                    if value >= 0 {
                                        offset = value as usize;
                                    }
                                }
                            }
                            let ti = table_index.unwrap_or(0) as usize;
                            while meta.tables.len() <= ti {
                                meta.tables.push(Vec::new());
                            }
                            let tab = &mut meta.tables[ti];
                            if tab.len() < offset + funcs.len() {
                                tab.resize(offset + funcs.len(), None);
                            }
                            for (i, f) in funcs.iter().enumerate() {
                                tab[offset + i] =
                                    if *f == u32::MAX { None } else { Some(*f) };
                            }
                        }
                        _ => {}
                    }
                }
            }
            Payload::CustomSection(reader) => {
                if let KnownCustom::Name(nr) = reader.as_known() {
                    for n in nr {
                        let n = n?;
                        if let Name::Function(map) = n {
                            for nm in map.into_iter() {
                                let nm = nm?;
                                meta.func_names.insert(nm.index, nm.name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(meta)
}

pub fn func_display_name(idx: u32, meta: &ModuleMeta) -> String {
    if let Some(n) = meta.func_names.get(&idx) {
        sanitize(n)
    } else if let Some(n) = meta.exports.get(&idx) {
        sanitize(n)
    } else {
        format!("f{idx}")
    }
}

fn sanitize(n: &str) -> String {
    let mut s: String = n
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        s = "f".into();
    }
    if s.chars().next().unwrap().is_numeric() {
        s = format!("f_{s}");
    }
    s
}
