use alloy_primitives::keccak256;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub selector: String,
    pub format_id: Option<String>,
    pub source_file: String,
    pub fields: Vec<SimpleField>,
}

#[derive(Debug, Clone)]
pub struct SimpleField {
    pub label: String,
    pub path: String,
    pub format: Option<String>,
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// Recursively visit a directory and invoke callback with (path, contents) for each UTF-8 file.
fn visit_dir<F: FnMut(&std::path::Path, &str)>(dir: &std::path::Path, cb: &mut F) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, cb);
            } else if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(s) = String::from_utf8(bytes) {
                    cb(&path, &s);
                }
            }
        }
    }
}

fn escape(s: &str) -> String {
    s.replace('"', "\\\"")
}

/// Normalize a format key into a 4-byte calldata selector (0xXXXXXXXX)
/// Accepted inputs:
/// - Already a selector: "0x0123abcd" (case-insensitive)
/// - Function signature: "transfer(address,uint256)" -> keccak256 and take first 4 bytes
/// Any other form (e.g., EIP-712 primary type like "mint") returns None.
pub fn normalize_selector(key: &str) -> Option<String> {
    let k = key.trim();
    // Already a 4-byte selector
    if k.len() == 10 && k.starts_with("0x") && k.chars().skip(2).all(|c| c.is_ascii_hexdigit()) {
        return Some(k.to_ascii_lowercase());
    }
    // Function signature form: name(args)
    if let (Some(_l), Some(r)) = (k.find('('), k.rfind(')')) {
        if r > 0 {
            let sig = &k[..=r]; // include ')'
            // Remove any internal whitespace to be safe
            let cleaned: String = sig.chars().filter(|c| !c.is_whitespace()).collect();
            let digest = keccak256(cleaned.as_bytes());
            let selector = &digest.as_slice()[..4];
            return Some(format!(
                "0x{:02x}{:02x}{:02x}{:02x}",
                selector[0], selector[1], selector[2], selector[3]
            ));
        }
    }
    None
}

/// Read the ERC-7730 registry under the given directory and collect display entries
/// for calldata (selector-keyed) formats. Primary parsing uses the adapter; if that
/// fails, a lightweight fallback extracts display.formats[*].fields labels/paths.
pub fn collect_entries(registry_dir: &Path) -> Vec<RegistryEntry> {
    let mut entries: Vec<RegistryEntry> = Vec::new();

    visit_dir(registry_dir, &mut |path, contents| {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext != "json" {
                return;
            }
        } else {
            return;
        }

        // Primary parse via adapter
        let mut parsed_any = false;
        if let Ok(spec) = visualsign_erc7730_adapter::types::ERC7730::from_json(contents) {
            if let Some(display) = spec.display {
                for (key, format) in display.formats.into_iter() {
                    if let Some(selector) = normalize_selector(&key) {
                        let fields: Vec<_> = format
                            .fields
                            .into_iter()
                            .map(|f| SimpleField {
                                label: f.label,
                                path: f.path,
                                format: None, // The adapter doesn't provide format info, will extract from raw JSON instead
                                params: None,
                            })
                            .collect();
                        entries.push(RegistryEntry {
                            selector,
                            format_id: format.id,
                            source_file: path
                                .strip_prefix(registry_dir)
                                .unwrap_or(path)
                                .to_string_lossy()
                                .to_string(),
                            fields,
                        });
                        parsed_any = true;
                    }
                }
            }
        }
        if !parsed_any {
            // Fallback lightweight parse of display.formats[*].fields
            #[derive(Deserialize)]
            struct FbField {
                label: Option<String>,
                path: Option<String>,
                #[serde(rename = "$ref")]
                r#ref: Option<String>,
                format: Option<String>,
                params: Option<HashMap<String, serde_json::Value>>,
            }
            #[derive(Deserialize)]
            struct FbFormat {
                #[serde(rename = "$id")]
                id: Option<String>,
                fields: Option<Vec<FbField>>,
            }
            #[derive(Deserialize)]
            struct FbDefinition {
                label: Option<String>,
                format: Option<String>,
                params: Option<HashMap<String, serde_json::Value>>,
            }
            #[derive(Deserialize)]
            struct FbDisplay {
                formats: HashMap<String, FbFormat>,
                definitions: Option<HashMap<String, FbDefinition>>,
            }
            #[derive(Deserialize)]
            struct FbSpec {
                display: Option<FbDisplay>,
            }
            if let Ok(fb) = serde_json::from_str::<FbSpec>(contents) {
                if let Some(display) = fb.display {
                    let defs = display.definitions.unwrap_or_default();
                    for (key, fmt) in display.formats.into_iter() {
                        if let Some(selector) = normalize_selector(&key) {
                            let fields: Vec<_> = fmt
                                .fields
                                .unwrap_or_default()
                                .into_iter()
                                .map(|f| {
                                    // derive label: explicit label, else from $ref -> definitions
                                    let label = if let Some(lbl) = f.label {
                                        lbl
                                    } else if let Some(ref r) = f.r#ref {
                                        let key = r.rsplit('.').next().unwrap_or(r);
                                        defs.get(key)
                                            .and_then(|d| d.label.clone())
                                            .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };

                                    // derive format: explicit format, else from $ref -> definitions
                                    let format = if let Some(fmt) = f.format {
                                        Some(fmt)
                                    } else if let Some(ref r) = f.r#ref {
                                        let key = r.rsplit('.').next().unwrap_or(r);
                                        defs.get(key).and_then(|d| d.format.clone())
                                    } else {
                                        None
                                    };

                                    // derive params: explicit params, else from $ref -> definitions
                                    let params = if let Some(p) = f.params {
                                        Some(p)
                                    } else if let Some(ref r) = f.r#ref {
                                        let key = r.rsplit('.').next().unwrap_or(r);
                                        defs.get(key).and_then(|d| d.params.clone())
                                    } else {
                                        None
                                    };

                                    SimpleField {
                                        label,
                                        path: f.path.unwrap_or_default(),
                                        format,
                                        params,
                                    }
                                })
                                .collect();
                            entries.push(RegistryEntry {
                                selector,
                                format_id: fmt.id,
                                source_file: path
                                    .strip_prefix(registry_dir)
                                    .unwrap_or(path)
                                    .to_string_lossy()
                                    .to_string(),
                                fields,
                            });
                        }
                    }
                }
            }
        }
    });

    entries
}

/// Generate the Rust source for the registry map used at runtime.
pub fn generate_registry_rs(entries: &[RegistryEntry]) -> String {
    // De-duplicate selectors grouping indexes
    let mut selector_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, e) in entries.iter().enumerate() {
        selector_map
            .entry(e.selector.clone())
            .or_default()
            .push(idx);
    }

    let mut out = String::new();
    out.push_str("// @generated automatically by build.rs; DO NOT EDIT\n\n");
    out.push_str(
        "#[derive(Debug)] pub struct GenField { pub label: &'static str, pub path: &'static str, pub format: Option<&'static str> }\n",
    );
    out.push_str("#[derive(Debug)] pub struct GenFormat { pub source_file: &'static str, pub selector: &'static str, pub format_id: Option<&'static str>, pub fields: &'static [GenField] }\n\n");

    // Emit fields and formats as separate static arrays for reuse
    for (i, entry) in entries.iter().enumerate() {
        out.push_str(&format!(
            "static FIELDS_{i}: [GenField; {}] = [",
            entry.fields.len()
        ));
        for f in &entry.fields {
            let format_str = f
                .format
                .as_ref()
                .map(|s| format!("Some(\"{}\")", escape(s)))
                .unwrap_or_else(|| "None".to_string());
            out.push_str(&format!(
                "GenField {{ label: \"{}\", path: \"{}\", format: {} }},",
                escape(&f.label),
                escape(&f.path),
                format_str
            ));
        }
        out.push_str("];\n\n");
        let format_id = entry
            .format_id
            .as_ref()
            .map(|s| format!("Some(\"{}\")", escape(s)))
            .unwrap_or_else(|| "None".to_string());
        out.push_str(&format!(
            "static FORMAT_{i}: GenFormat = GenFormat {{ source_file: \"{}\", selector: \"{}\", format_id: {format_id}, fields: &FIELDS_{i} }};\n\n",
            escape(&entry.source_file),
            escape(&entry.selector)
        ));
    }

    // Build per-selector format slices
    let mut grouped: Vec<(&String, &Vec<usize>)> = selector_map.iter().collect();
    grouped.sort_by(|a, b| a.0.cmp(b.0));
    for (idx, (_sel, list)) in grouped.iter().enumerate() {
        out.push_str(&format!(
            "static FORMATS_FOR_{idx}: [&GenFormat; {}] = [",
            list.len()
        ));
        for fi in *list {
            out.push_str(&format!("&FORMAT_{fi},"));
        }
        out.push_str("];\n\n");
    }

    // phf map: selector -> slice of &GenFormat
    out.push_str(
        "pub static SELECTOR_MAP: phf::Map<&'static str, &'static [&'static GenFormat]> = phf::phf_map! {\n",
    );
    for (idx, (sel, _)) in grouped.iter().enumerate() {
        out.push_str(&format!("    \"{}\" => &FORMATS_FOR_{idx},\n", escape(sel)));
    }
    out.push_str("};\n");
    out
}
