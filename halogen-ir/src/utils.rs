use std::path::Path;
use std::{fs, io};

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

pub fn create_dir_if_not_exist(path: &Path) -> io::Result<()> {
    match fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}
