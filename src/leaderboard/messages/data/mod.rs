pub mod get_rank;

pub mod upsert_elements;

// Common traits and enums

#[repr(i32)]
pub enum Order {
    Ascending = 0,
    Descending = 1,
}

/// This trait defines an interface for converting a type into a vector of [SortedSetElement].
pub trait IntoIds: Send {
    /// Converts the type into a vector of [SortedSetElement].
    fn into_ids(self) -> Vec<u32>;
}

#[cfg(not(doctest))]
pub fn map_and_collect_elements<'a, I>(iter: I) -> Vec<u32>
where
    I: Iterator<Item = &'a u32>,
{
    iter.copied().collect()
}

impl IntoIds for Vec<u32> {
    fn into_ids(self) -> Vec<u32> {
        self
    }
}

impl IntoIds for &[u32] {
    fn into_ids(self) -> Vec<u32> {
        map_and_collect_elements(self.iter())
    }
}
