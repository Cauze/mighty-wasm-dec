use std::collections::{BTreeMap, HashMap};

use crate::ir::{FuncIR, MemAccess};

#[derive(Debug, Clone)]
pub struct Field {
    pub offset: u64,
    pub width: u8,
    pub dom_ty: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct StructLayout {
    pub name: String,
    pub base: String,
    pub fields: Vec<Field>,
    /// true if base looks like a heap pointer (vs stack/global const)
    pub is_pointer: bool,
}

fn width_to_c(width: u8, dom: &str) -> &'static str {
    match (width, dom) {
        (1, _) => "int8_t",
        (2, _) => "int16_t",
        (4, "f32") => "float",
        (4, _) => "int32_t",
        (8, "f64") => "double",
        (8, "f64/w8") => "double",
        (8, _) => "int64_t",
        (16, _) => "__v128",
        _ => "int32_t",
    }
}

/// Per-function struct recovery: group accesses by base_desc.
pub fn recover(func: &FuncIR) -> Vec<StructLayout> {
    let mut groups: BTreeMap<String, Vec<&MemAccess>> = BTreeMap::new();
    for a in &func.accesses {
        if a.base_desc == "expr" {
            continue;
        }
        groups.entry(a.base_desc.clone()).or_default().push(a);
    }
    let mut out = Vec::new();
    for (base, accs) in groups {
        if accs.len() < 1 {
            continue;
        }
        // merge by offset: keep widest + dominant type by count
        let mut by_off: BTreeMap<u64, Vec<&MemAccess>> = BTreeMap::new();
        for a in accs {
            by_off.entry(a.offset).or_default().push(a);
        }
        let mut fields: Vec<Field> = Vec::new();
        for (off, v) in by_off {
            let width = v.iter().map(|a| a.width).max().unwrap_or(4);
            // dominant type
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for a in &v {
                *counts.entry(a.dom_ty.as_str()).or_default() += 1;
            }
            let dom = counts
                .into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(t, _)| t.to_string())
                .unwrap_or("i32".into());
            fields.push(Field {
                offset: off,
                width,
                dom_ty: dom,
                count: v.len(),
            });
        }
        // only report if it looks struct-like: >=2 distinct offsets, or repeated same base
        if fields.len() >= 2 || func.accesses.iter().filter(|a| a.base_desc == base).count() >= 3 {
            out.push(StructLayout {
                name: format!("S_{}_{}", func.idx, base),
                base,
                fields,
                is_pointer: true,
            });
        }
    }
    out
}

pub fn field_c_ty(f: &Field) -> &'static str {
    width_to_c(f.width, &f.dom_ty)
}

/// Cross-function merge with union-find over shared offsets (bounded).
/// Returns merged struct names for display; MVP keeps per-func structs
/// and only reports merges as comments.
pub fn merge_notes(funcs: &[FuncIR]) -> Vec<String> {
    // parent map over (func_idx, base) -> root
    let mut notes = Vec::new();
    let mut sig_map: HashMap<Vec<u64>, Vec<String>> = HashMap::new();
    for f in funcs {
        let layouts = recover(f);
        for l in layouts {
            let mut sig: Vec<u64> = l.fields.iter().map(|x| x.offset).collect();
            sig.sort();
            sig_map.entry(sig.clone()).or_default().push(format!("f{}:{}", f.idx, l.base));
        }
    }
    for (sig, owners) in sig_map {
        if owners.len() > 1 && owners.len() <= 8 {
            notes.push(format!(
                "/* merge hint: same layout {:?} shared by {} */",
                sig,
                owners.join(", ")
            ));
        }
    }
    notes
}
