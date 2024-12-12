use std::collections::HashMap;
use std::env::var_os;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};

use anyhow::{bail, Context as _, Result};
use heck::*;
use tera::Tera;

use crate::ir;
use crate::utils::into_maybe_par_iter;
use crate::utils::rayon_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Utils {
    Super,
    Embed,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Rustfmt,
    None,
}

fn create_dir(path: &Path) -> io::Result<()> {
    match fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

pub fn gen_multi_chip(
    multi: &ir::MultiChip,
    root: &Path,
    _utils: Utils,
    format: Format,
) -> Result<()> {
    // First create necessary directories
    create_dir(&root.join("chips")).context("failed to create chips dir")?;
    create_dir(&root.join("modules")).context("failed to create modules dir")?;

    into_maybe_par_iter(&multi.chips).try_for_each(|chip| -> Result<()> {
        let path = root
            .join("chips")
            .join(format!("{}.rs", chip.name.to_lowercase()));
        let out = fs::File::create(path).context("failed to create chip file")?;

        if chip.modules.is_empty() {
            gen_chip(chip, Some(".."), Utils::None, format, out)
        } else {
            gen_chip(chip, Some(".."), Utils::Super, format, out)
        }
    })?;

    into_maybe_par_iter(&multi.modules).try_for_each(|module| -> Result<()> {
        let Some(version) = &module.version else {
            return Ok(());
        };

        let path = root
            .join("modules")
            .join(format!("{}_{}.rs", module.name, version));
        let out = fs::File::create(path).context("failed to create module file")?;

        gen_module(module, Utils::Super, format, out)
    })?;

    Ok(())
}

pub fn gen_chip(
    chip: &ir::Chip,
    root: Option<&str>,
    utils: Utils,
    format: Format,
    out: impl Write,
) -> Result<()> {
    let tera = tera()?;

    let mut ctx = tera::Context::new();
    ctx.insert("chip", chip);
    ctx.insert("root", &root);
    ctx.insert(
        "utils",
        match utils {
            Utils::Super => "super",
            Utils::Embed => "embed",
            Utils::None => "none",
        },
    );

    render_with_fmt(&tera, "chip.tera", &ctx, format, out)
}

pub fn gen_module(
    module: &ir::Module,
    utils: Utils,
    format: Format,
    out: impl Write,
) -> Result<()> {
    let tera = tera()?;

    let mut ctx = tera::Context::new();
    ctx.insert("module", module);
    ctx.insert(
        "utils",
        match utils {
            Utils::Super => "super",
            Utils::Embed => "embed",
            Utils::None => "none",
        },
    );

    render_with_fmt(&tera, "module.tera", &ctx, format, out)
}

fn render_with_fmt(
    tera: &Tera,
    file: &str,
    ctx: &tera::Context,
    format: Format,
    out: impl Write,
) -> Result<()> {
    match format {
        Format::Rustfmt => run_with_rustfmt(
            |out| {
                tera.render_to(file, ctx, out).context("failed to render")?;
                Ok(())
            },
            out,
        ),
        Format::None => {
            let s = tera.render_to(file, ctx, out).context("failed to render")?;
            Ok(s)
        }
    }
}

fn tera() -> Result<Tera> {
    let mut tera = super::tera();

    fn stringify(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("stringify", "value", String, v);
        Ok(format!("{:?}", s).into())
    }

    fn escape_keyword(s: String) -> String {
        match s.as_str() {
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern"
            | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod"
            | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct"
            | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"
            | "async" | "await" | "dyn" | "abstract" | "become" | "box" | "do" | "final"
            | "macro" | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield"
            | "try" => format!("{s}_"),
            _ => s,
        }
    }

    fn to_mod_name(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("to_mod_name", "value", String, v);
        Ok(escape_keyword(s.to_snake_case()).into())
    }

    fn to_type_name(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("to_type_name", "value", String, v);
        Ok(escape_keyword(s.to_upper_camel_case()).into())
    }

    fn to_fn_name(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("to_fn_name", "value", String, v);
        Ok(escape_keyword(s.to_snake_case()).into())
    }

    fn to_const_name(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("to_const_name", "value", String, v);
        Ok(escape_keyword(s.to_shouty_snake_case()).into())
    }

    tera.register_filter("stringify", stringify);

    tera.register_filter("to_mod_name", to_mod_name);
    tera.register_filter("to_type_name", to_type_name);
    tera.register_filter("to_fn_name", to_fn_name);
    tera.register_filter("to_const_name", to_const_name);

    tera.add_raw_templates([
        ("utils.rs", include_str!("rust/templates/utils.rs")),
        ("chip.tera", include_str!("rust/templates/chip.tera")),
        ("macro.tera", include_str!("rust/templates/macro.tera")),
        ("module.tera", include_str!("rust/templates/module.tera")),
        ("block.tera", include_str!("rust/templates/block.tera")),
        (
            "bitfield.tera",
            include_str!("rust/templates/bitfield.tera"),
        ),
        ("enum.tera", include_str!("rust/templates/enum.tera")),
    ])?;

    Ok(tera)
}

fn run_with_rustfmt<F>(f: F, mut out: impl Write) -> Result<()>
where
    F: FnOnce(&mut ChildStdin) -> Result<()>,
{
    let rustfmt = var_os("RUSTFMT").unwrap_or_else(|| From::from("rustfmt"));

    let mut child = Command::new(rustfmt)
        .args(&[
            "--emit",
            "stdout",
            "--color",
            "never",
            "--config",
            "blank_lines_upper_bound=0",
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn rustfmt process")?;

    let mut stdin = child.stdin.take().unwrap();
    f(&mut stdin)?;

    // Close stdin
    let _ = stdin.flush();
    drop(stdin);

    let output = child
        .wait_with_output()
        .context("rustfmt didn't exit gracefully")?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).context("rustfmt didn't output utf8")?;
        bail!("rustfmt failed, stderr: {stderr}")
    }

    out.write_all(&output.stdout)
        .context("failed to write output")?;
    Ok(())
}
