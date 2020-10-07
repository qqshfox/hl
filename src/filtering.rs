use std::{collections::HashSet, marker::PhantomData};

// ---

pub trait AnyKeyMatcher<'a>: Sized {
    type Key: From<&'a str> + Into<&'a str> + 'a;
    type Options: Sized + Copy;
    fn new(key: Self::Key, options: Self::Options) -> Self;
    fn match_key<M: AnyKeyMatcher<'a>>(&'a self, key: &str) -> Option<KeyMatch<'a, M>>;
}

// ---
/*
pub trait AnyKeyMatcherFactory<'a, M>: Sized
where
    M: AnyKeyMatcher + 'a,
{
    fn new_key_matcher(&self, key: &'a str) -> M;
}
*/
// ---

#[derive(Debug)]
pub enum KeyMatch<'a, M>
where
    M: AnyKeyMatcher<'a>,
{
    Full,
    Partial(M),
}

// ---

#[derive(Debug, Copy, Clone)]
pub struct KeyMatcherOptions<N> {
    pub delimiter: u8,
    pub norm: N,
}

// ---

#[derive(Debug)]
pub struct KeyMatcher<'a, N> {
    key: &'a str,
    options: KeyMatcherOptions<N>,
}

impl<'a, N> KeyMatcher<'a, N>
where
    N: Fn(u8) -> u8 + Copy,
{
    pub fn new(key: &'a str, options: KeyMatcherOptions<N>) -> Self {
        Self { key, options }
    }
}

impl<'a, N> AnyKeyMatcher<'a> for KeyMatcher<'a, N>
where
    N: Fn(u8) -> u8 + Copy,
{
    type Key = &'a str;
    type Options = KeyMatcherOptions<N>;

    fn new(key: Self::Key, options: Self::Options) -> Self {
        Self { key, options }
    }

    fn match_key<M: AnyKeyMatcher<'a>>(&self, key: &str) -> Option<KeyMatch<M>> {
        let norm = &self.options.norm;
        let bytes = self.key.as_bytes();
        if bytes
            .iter()
            .zip(key.as_bytes().iter())
            .position(|(&x, &y)| norm(x) != norm(y))
            .is_some()
        {
            return None;
        }

        if self.key.len() == key.len() {
            Some(KeyMatch::Full)
        } else if self.key.len() > key.len() {
            if bytes[key.len()] == self.options.delimiter {
                Some(KeyMatch::Partial(M::new(
                    &self.key[key.len() + 1..].into(),
                    self.options,
                )))
            } else {
                None
            }
        } else {
            None
        }
    }
}

// ---
/*
#[derive(Debug)]
pub struct KeyMatcherFactory<N> {
    delimiter: u8,
    norm: N,
}

impl<N> KeyMatcherFactory<N>
where
    N: Fn(u8) -> u8 + Copy,
{
    pub fn new(delimiter: u8, norm: N) -> Self {
        Self { delimiter, norm }
    }
}

impl<'a, N> AnyKeyMatcherFactory<'a, KeyMatcher<'a, N>> for KeyMatcherFactory<N>
where
    N: Fn(u8) -> u8 + Copy + 'a,
{
    fn new_key_matcher(&self, key: &'a str) -> KeyMatcher<'a, N> {
        KeyMatcher::new(key, self.delimiter, self.norm)
    }
}
*/
// ---

pub struct IncludeExcludeKeyMatcher<O> {
    include: HashSet<String>,
    exclude: HashSet<String>,
    options: O,
}

impl<O> IncludeExcludeKeyMatcher<O> {
    pub fn new(options: O) -> Self {
        Self {
            include: HashSet::new(),
            exclude: HashSet::new(),
            options,
        }
    }

    pub fn include(&mut self, key: String) -> &mut Self {
        self.include.insert(key);
        self
    }

    pub fn exclude(&mut self, key: String) -> &mut Self {
        self.exclude.insert(key);
        self
    }
}

impl<O> AnyKeyMatcher<'static> for IncludeExcludeKeyMatcher<O>
where
    M: AnyKeyMatcher<'a>,
{
    fn match_key(&self, key: &str) -> Option<KeyMatch<'a, Self>> {
        match self.include.get(key) {
            None => None,
            Some(k) => {
                let km: Self = self.factory.new_key_matcher(k);
                km.match_key(key)
            }
        }
    }
}
