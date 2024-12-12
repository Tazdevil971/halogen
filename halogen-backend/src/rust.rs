use std::borrow::Cow;
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

fn to_tera_utils(utils: Utils) -> &'static str {
    match utils {
        Utils::Super => "super",
        Utils::Embed => "embed",
        Utils::None => "none",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Rustfmt,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenMultiChipSettings<'a> {
    pub utils: Utils,
    pub format: Format,
    pub core_path: Option<&'a str>,
    pub gen_chips: bool,
    pub gen_list: bool,
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
        settings: GenMultiChipSettings<'_>,
    ) -> io::Result<()> {
        let root = root.as_ref();
        let chips_path = root.join("chips");
        let modules_path = root.join("modules");

        // Remove trailing / in core_path
        let core_path = settings.core_path.map(|path| path.trim_end_matches("/"));

        // First create necessary directories
        utils::create_dir_if_not_exist(&root)?;
        utils::create_dir_if_not_exist(&chips_path)?;
        utils::create_dir_if_not_exist(&modules_path)?;

        let (res1, (res2, (res3, res4))) = utils::maybe_par_multi_join! {
            || {
                if settings.gen_list {
                    let mut ctx = tera::Context::new();
                    ctx.insert("chips", &multi.chips);

                    let path = root.join("list.rs");
                    let out = io::BufWriter::new(fs::File::create(path)?);

                    render_with_fmt(&self.tera, "list.tera", &ctx, settings.format, out)
                } else {
                    Ok(())
                }
            },
            || {
                if settings.gen_chips {
                    let mut ctx = tera::Context::new();
                    ctx.insert("chips", &multi.chips);
                    ctx.insert("root", ".");
                    ctx.insert("core_path", &core_path);
                    ctx.insert("utils", to_tera_utils(settings.utils));

                    let path = root.join("chips.rs");
                    let out = io::BufWriter::new(fs::File::create(path)?);

                    render_with_fmt(&self.tera, "chips.tera", &ctx, settings.format, out)
                } else {
                    Ok(())
                }
            },
            || {
                utils::into_maybe_par_iter(&multi.chips).try_for_each(
                    |chip| -> io::Result<()> {
                        let name = escape_keyword(chip.name.to_snake_case().into());

                        let path = chips_path.join(format!("{name}.rs"));
                        let out = io::BufWriter::new(fs::File::create(path)?);

                        self.gen_chip(chip, Some(".."), Utils::Super, settings.format, out)
                    },
                )
            },
            || {
                utils::into_maybe_par_iter(&multi.modules).try_for_each(
                    |module| -> io::Result<()> {
                        let name = if let Some(version) = &module.version {
                            Cow::Owned(format!("{}_{}", module.name, version))
                        } else {
                            Cow::Borrowed(&module.name)
                        };

                        let name = escape_keyword(name.to_snake_case().into());

                        let path = modules_path.join(format!("{name}.rs"));
                        let out = io::BufWriter::new(fs::File::create(path)?);

                        self.gen_module(module, Utils::Super, settings.format, out)
                    },
                )
            }
        };

        res1?;
        res2?;
        res3?;
        res4?;

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
        ctx.insert("utils", to_tera_utils(utils));

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
        ctx.insert("utils", to_tera_utils(utils));

        render_with_fmt(&self.tera, "module.tera", &ctx, format, out)
    }
}

fn escape_keyword(s: Cow<'_, str>) -> Cow<'_, str> {
    match s.as_ref() {
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move"
        | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await"
        | "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" => {
            Cow::Owned(format!("{s}_"))
        }
        _ => s,
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

    fn escape_keyword2(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("stringify", "value", String, v);
        let s = escape_keyword(s.into());
        Ok(s.into())
    }

    tera.register_filter("stringify", stringify);
    tera.register_filter("escape_keyword", escape_keyword2);

    tera.add_raw_templates([
        ("utils.rs", include_str!("rust/templates/utils.rs")),
        ("chip.tera", include_str!("rust/templates/chip.tera")),
        ("macro.tera", include_str!("rust/templates/macro.tera")),
        (
            "peripheral.tera",
            include_str!("rust/templates/peripheral.tera"),
        ),
        ("module.tera", include_str!("rust/templates/module.tera")),
        ("block.tera", include_str!("rust/templates/block.tera")),
        (
            "bitfield.tera",
            include_str!("rust/templates/bitfield.tera"),
        ),
        ("enum.tera", include_str!("rust/templates/enum.tera")),
        ("cm_reg.tera", include_str!("rust/templates/cm_reg.tera")),
        ("chips.tera", include_str!("rust/templates/chips.tera")),
        ("list.tera", include_str!("rust/templates/list.tera")),
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
            tera.render_to(file, ctx, out)
                .map_err(utils::unwrap_tera_error)?;
            Ok(())
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

        return Err(io::Error::other(format!("rustfmt failed with:\n{stderr}")));
    }

    out.write_all(&output.stdout)?;
    Ok(())
}
