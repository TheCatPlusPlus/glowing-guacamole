use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::anyhow;
use unicode_ident::{is_xid_start, is_xid_continue};

fn get_crate_path() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .expect("No CARGO_MANIFEST_DIR")
        .into()
}

fn to_identifier(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 1);

    if let Some(first) = name.chars().next() {
        if !is_xid_start(first) {
            result.push('_');
        }

        for ch in name.chars() {
            result.push(if is_xid_continue(ch) { ch } else { '_' });
        }
    }

    result
}

fn to_literal(name: &str) -> String {
    format!("r#\"{}\"#", name)
}

fn generate() -> anyhow::Result<()> {
    let source_path = get_crate_path().join("src");
    let generated_path = source_path.join("generated.rs");

    let cell_id_path = source_path.join("cellID.txt");
    let entity_id_path = source_path.join("entityID.txt");
    let item_id_path = source_path.join("itemID.txt");
    let prop_id_path = source_path.join("propID.txt");

    println!("cargo:rerun-if-changed={}", cell_id_path.display());
    println!("cargo:rerun-if-changed={}", entity_id_path.display());
    println!("cargo:rerun-if-changed={}", item_id_path.display());
    println!("cargo:rerun-if-changed={}", prop_id_path.display());

    let mut target = File::create(&generated_path)?;
    let items = {
        let mut vec: Vec<(i32, String)> = Vec::new();

        for line in fs::read_to_string(item_id_path)?.lines() {
            let mut parts = line.trim().splitn(2, " ");

            let id_str = parts
                .next()
                .ok_or(anyhow!("Bad format (id): {line}"))?;

            let name = parts.next().ok_or(anyhow!("Bad format (name): {line}"))?;

            let id = id_str
                .parse::<i32>()
                .or(Err(anyhow!("Failed to parse: {id_str}")))?;


            vec.push((id, name.into()));
        }

        vec.sort_by_key(|k| k.0);
        vec
    };

    let sp = "    ";

    writeln!(target, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    writeln!(target, "#[allow(non_camel_case_types)]")?;
    writeln!(target, "pub enum ItemId {{")?;

    for (_, name) in &items {
        writeln!(target, "{sp}{},", to_identifier(name))?;
    }

    writeln!(target, "}}")?;
    writeln!(target)?;

    writeln!(target, "impl ItemId {{")?;

    writeln!(target, "{sp}pub fn from_id(id: i32) -> Option<Self> {{")?;
    writeln!(target, "{sp}{sp}match id {{")?;
    for (id, name) in &items {
        writeln!(target, "{sp}{sp}{sp}{} => Some(Self::{}),", id, to_identifier(name))?;
    }
    writeln!(target, "{sp}{sp}{sp}_ => None,")?;
    writeln!(target, "{sp}{sp}}}")?;
    writeln!(target, "{sp}}}")?;
    writeln!(target)?;

    writeln!(target, "{sp}pub fn id(&self) -> i32 {{")?;
    writeln!(target, "{sp}{sp}match self {{")?;
    for (id, name) in &items {
        writeln!(target, "{sp}{sp}{sp}Self::{} => {},", to_identifier(name), id)?;
    }
    writeln!(target, "{sp}{sp}}}")?;
    writeln!(target, "{sp}}}")?;
    writeln!(target)?;

    writeln!(target, "{sp}pub fn name(&self) -> &'static str {{")?;
    writeln!(target, "{sp}{sp}match self {{")?;
    for (id, name) in &items {
        writeln!(target, "{sp}{sp}{sp}Self::{} => {},", to_identifier(name), to_literal(name))?;
    }
    writeln!(target, "{sp}{sp}}}")?;
    writeln!(target, "{sp}}}")?;
    writeln!(target, "}}")?;
    writeln!(target)?;

    writeln!(target, "impl TryFrom<i32> for ItemId {{")?;
    writeln!(target, "{sp}type Error = &'static str;")?;
    writeln!(target, "{sp}fn try_from(id: i32) -> Result<Self, Self::Error> {{")?;
    writeln!(target, "{sp}{sp}Self::from_id(id).ok_or(\"unknown id\")")?;
    writeln!(target, "{sp}}}")?;
    writeln!(target, "}}")?;
    writeln!(target)?;

    writeln!(target, "impl Into<i32> for ItemId {{")?;
    writeln!(target, "{sp}fn into(self) -> i32 {{")?;
    writeln!(target, "{sp}{sp}self.id()")?;
    writeln!(target, "{sp}}}")?;
    writeln!(target, "}}")?;
    writeln!(target)?;

    writeln!(target, "impl std::fmt::Display for ItemId {{")?;
    writeln!(target, "{sp}fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{")?;
    writeln!(target, "{sp}{sp}f.write_str(self.name())")?;
    writeln!(target, "{sp}}}")?;
    writeln!(target, "}}")?;

    Ok(())
}

fn main() {
    generate().expect("Failed to generate the stuff");
}
