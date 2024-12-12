use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};

use halogen_ir::ir;
use heck::*;
use tera::Tera;

use crate::utils;
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

pub struct GenCtx {
    tera: Tera,
}

impl Default for GenCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl GenCtx {
    pub fn new() -> Self {
        Self { tera: tera() }
    }

    pub fn gen_multi_chip(
        &self,
        multi: &ir::MultiChip,
        root: impl AsRef<Path>,
        _utils: Utils,
        format: Format,
    ) -> io::Result<()> {
        let root = root.as_ref();
        let chips_path = root.join("chips");
        let modules_path = root.join("modules");

        // First create necessary directories
        utils::create_dir_if_not_exist(&chips_path)?;
        utils::create_dir_if_not_exist(&modules_path)?;

        let (res1, res2) = utils::maybe_par_join(
            || {
                utils::into_maybe_par_iter(&multi.chips).try_for_each(|chip| -> io::Result<()> {
                    let path = chips_path.join(format!("{}.rs", chip.name.to_lowercase()));
                    let out = fs::File::create(path)?;

                    self.gen_chip(chip, Some(".."), Utils::None, format, out)
                })
            },
            || {
                utils::into_maybe_par_iter(&multi.modules).try_for_each(
                    |module| -> io::Result<()> {
                        let Some(version) = &module.version else {
                            return Ok(());
                        };

                        let path = modules_path.join(format!("{}_{}.rs", module.name, version));
                        let out = fs::File::create(path)?;

                        self.gen_module(module, Utils::Super, format, out)
                    },
                )
            },
        );

        res1?;
        res2?;

        Ok(())
    }

    pub fn gen_chip(
        &self,
        chip: &ir::Chip,
        root: Option<&str>,
        utils: Utils,
        format: Format,
        out: impl Write,
    ) -> io::Result<()> {
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

        render_with_fmt(&self.tera, "chip.tera", &ctx, format, out)
    }

    pub fn gen_module(
        &self,
        module: &ir::Module,
        utils: Utils,
        format: Format,
        out: impl Write,
    ) -> io::Result<()> {
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

        render_with_fmt(&self.tera, "module.tera", &ctx, format, out)
    }
}

fn tera() -> Tera {
    let mut tera = utils::tera();

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
    ])
    .expect("Failed to compile tera templates");

    tera
}

fn render_with_fmt(
    tera: &Tera,
    file: &str,
    ctx: &tera::Context,
    format: Format,
    out: impl Write,
) -> io::Result<()> {
    // TODO: Better handle Tera errors
    match format {
        Format::Rustfmt => run_with_rustfmt(
            |out| {
                tera.render_to(file, ctx, out)
                    .map_err(utils::unwrap_tera_error)?;
                Ok(())
            },
            out,
        ),
        Format::None => {
            let s = tera
                .render_to(file, ctx, out)
                .map_err(utils::unwrap_tera_error)?;
            Ok(s)
        }
    }
}

fn run_with_rustfmt<F>(f: F, mut out: impl Write) -> io::Result<()>
where
    F: FnOnce(&mut ChildStdin) -> io::Result<()>,
{
    let rustfmt = std::env::var_os("RUSTFMT").unwrap_or_else(|| From::from("rustfmt"));

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
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    f(&mut stdin)?;

    // Close stdin
    let _ = stdin.flush();
    drop(stdin);

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let Ok(stderr) = String::from_utf8(output.stderr) else {
            return Err(io::Error::other("rustfmt outputted non unicode characters"));
        };

        return Err(io::Error::other(format!("rustfmt failed with {stderr:?}")));
    }

    out.write_all(&output.stdout)?;
    Ok(())
}
