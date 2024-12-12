use std::env::var_os;
use std::io::Write as _;
use std::process::{Command, Stdio};

use crate::ir;

use anyhow::{bail, Context as _, Result};
use tera::Tera;

pub fn gen_block(ir: &ir::Block) -> Result<String> {
    let tera = tera()?;

    let mut ctx = tera::Context::new();
    ctx.insert("ir", ir);
    ctx.insert("utils", "crate::utils");

    // run_with_rustfmt(&tera, "lib.rs.tera", &ctx)
    let output = tera.render("lib.rs.tera", &ctx).context("failed to spawn rustfmt process")?;
    Ok(remove_whitespace(&output))
}

fn tera() -> Result<Tera> {
    let mut tera = super::tera();
    tera.add_raw_templates([
        ("lib.rs.tera", include_str!("rust/templates/lib.rs.tera")),
        ("macro.tera", include_str!("rust/templates/macro.tera")),
        ("register.tera", include_str!("rust/templates/register.tera")),
        ("block.tera", include_str!("rust/templates/block.tera")),
    ])?;

    Ok(tera)
}

fn remove_whitespace(s: &str) -> String {
    let mut iter = s.split_ascii_whitespace();
    let mut acc = iter.next().map(String::from).unwrap_or_default();
    for item in iter {
        acc.push(' ');
        acc.push_str(item);
    }

    acc
}

fn run_with_rustfmt(tera: &Tera, file: &str, ctx: &tera::Context) -> Result<String> {
    let rustfmt = var_os("RUSTFMT").unwrap_or_else(|| From::from("rustfmt"));

    let mut child = Command::new(rustfmt)
        .args(&[
            "--emit", "stdout", 
            "--color", "never",
            "--config", "blank_lines_upper_bound=0"
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn rustfmt process")?;

    let mut stdin = child.stdin.take().unwrap();
    tera.render_to(file, ctx, &mut stdin)
        .context("failed to render template")?;

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

    let stdout = String::from_utf8(output.stdout).context("rustfmt didn't output utf8")?;
    Ok(stdout)
}
