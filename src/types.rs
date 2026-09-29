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
        (1, d) if d.ends_with("u8") => "uint8_t",
        (1, _) => "int8_t",
        (2, d) if d.ends_with("u16") => "uint16_t",
        (2, _) => "int16_t",
        (4, "f32") => "float",
        (4, d) if d.ends_with("u8") || d.ends_with("u16") || d.ends_with("u32") => "uint32_t",
        (4, _) => "int32_t",
        (8, "f64") => "double",
        (8, "f64/w8") => "double",
        (8, d) if d.ends_with("u32") || d.ends_with("u64") => "uint64_t",
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
        if accs.is_empty() {
            continue;
        }
        let n_accs = accs.len();
        // merge by offset: keep widest + dominant type by count
        let mut by_off: BTreeMap<u64, Vec<&MemAccess>> = BTreeMap::new();
        for a in accs {
            by_off.entry(a.offset).or_default().push(a);
        }
        let mut fields: Vec<Field> = Vec::new();
        for (off, v) in by_off {
            // Dominant width wins (NOT max): max-widening rendered narrow
            // accesses through wide fields (fill_blob's i32 tag store via
            // an int64_t f_off_0 overlapping f_off_4). Minority-width
            // accesses fall back to raw mem rendering at emit (width gate).
            let mut wcounts: HashMap<u8, usize> = HashMap::new();
            for a in &v {
                *wcounts.entry(a.width).or_default() += 1;
            }
            let width = wcounts
                .into_iter()
                .max_by(|(w1, c1), (w2, c2)| c1.cmp(c2).then(w2.cmp(w1)))
                .map(|(w, _)| w)
                .unwrap_or(4);
            // dominant type among accesses AT the dominant width
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for a in &v {
                if a.width == width {
                    *counts.entry(a.dom_ty.as_str()).or_default() += 1;
                }
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
        // Prune overlapping fields (keep earliest; ties keep narrower):
        // overlapping typedef ranges mislead and mis-render.
        fields.sort_by_key(|f| (f.offset, f.width));
        let mut kept: Vec<Field> = Vec::new();
        let mut max_end: u64 = 0;
        for fl in fields {
            if fl.offset >= max_end {
                max_end = fl.offset + fl.width as u64;
                kept.push(fl);
            }
        }
        fields = kept;
        // only report if it looks struct-like: >=2 distinct offsets, or repeated same base
        if fields.len() >= 2 || n_accs >= 3 {
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
