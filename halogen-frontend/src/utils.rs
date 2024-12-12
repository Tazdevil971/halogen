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
