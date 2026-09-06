use std::fmt::Display;

pub fn max_by_key<'a, T: Ord, F: Fn(&'a T) -> T>(
    items: &'a [T],
    key: F,
) -> Option<&'a T> {
    items.iter().max_by_key(key)
}

pub struct Container<T> {
    inner: Vec<T>,
}

impl<T: Display + Clone> Container<T> {
    pub fn to_strings(&self) -> Vec<String> {
        self.inner.iter().map(|x| x.to_string()).collect()
    }
}

pub fn setup() -> impl Iterator<Item = u32> {
    (0..10).map(|x| x * 2)
}
