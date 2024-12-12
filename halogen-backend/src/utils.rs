use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

use heck::*;
use tera::Tera;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub mod rayon_prelude {
    #[cfg(feature = "rayon")]
    pub use rayon::prelude::*;
}

#[cfg(feature = "rayon")]
pub fn into_maybe_par_iter<T, U>(c: T) -> impl ParallelIterator<Item = U>
where
    T: IntoParallelIterator<Item = U>,
    U: Send,
{
    c.into_par_iter()
}

#[cfg(not(feature = "rayon"))]
pub fn into_maybe_par_iter<T, U>(c: T) -> impl Iterator<Item = U>
where
    T: IntoIterator<Item = U>,
    U: Send,
{
    c.into_iter()
}

#[cfg(feature = "rayon")]
pub fn maybe_par_join<L, R, LR, RR>(l: L, r: R) -> (LR, RR)
where
    L: FnOnce() -> LR + Send,
    R: FnOnce() -> RR + Send,
    LR: Send,
    RR: Send,
{
    rayon::join(l, r)
}

#[cfg(not(feature = "rayon"))]
pub fn maybe_par_join<L, R, LR, RR>(l: L, r: R) -> (LR, RR)
where
    L: FnOnce() -> LR + Send,
    R: FnOnce() -> RR + Send,
    LR: Send,
    RR: Send,
{
    (l(), r())
}

#[macro_export]
macro_rules! maybe_par_multi_join {
    ($l:expr, $r:expr) => {
        $crate::utils::maybe_par_join($l, $r)
    };
    ($l:expr $(, $r:expr)+) => {
        $crate::utils::maybe_par_join(
            $l,
            || $crate::utils::maybe_par_multi_join!($($r),*)
        )
    };
}

pub use maybe_par_multi_join;

pub fn create_dir_if_not_exist(path: &Path) -> io::Result<()> {
    match fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

pub fn tera() -> Tera {
    let mut tera = Tera::default();

    fn upper_camel_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("upper_camel_case", "value", String, v);
        Ok(s.to_upper_camel_case().into())
    }

    fn lower_camel_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("lower_camel_case", "value", String, v);
        Ok(s.to_lower_camel_case().into())
    }

    fn shouty_snake_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("shouty_snake_case", "value", String, v);
        Ok(s.to_shouty_snake_case().into())
    }

    fn snake_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("snake_case", "value", String, v);
        Ok(s.to_snake_case().into())
    }

    fn hex(v: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let n = tera::try_get_value!("hex", "value", u64, v);
        Ok(format!("{n:#x}").into())
    }

    fn mask(v: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let n = tera::try_get_value!("mask", "value", u64, v);
        Ok(((1u64 << n) - 1).into())
    }

    tera.register_filter("upper_camel_case", upper_camel_case);
    tera.register_filter("lower_camel_case", lower_camel_case);
    tera.register_filter("shouty_snake_case", shouty_snake_case);
    tera.register_filter("snake_case", snake_case);
    tera.register_filter("hex", hex);
    tera.register_filter("mask", mask);

    tera
}

pub fn unwrap_tera_error(error: tera::Error) -> io::Error {
    match error.kind {
        tera::ErrorKind::Io(kind) => io::Error::new(kind, "tera IO error"),
        _ => panic!("{}", error),
    }
}
