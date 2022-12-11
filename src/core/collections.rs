use crate::core::{base_types::UTF8StringPair, properties::UserProperty};
use std::{fmt, str};

#[derive(Clone, Debug, Default)]
pub struct UserProperties {
    map: Vec<UTF8StringPair>,
}

impl From<Vec<UTF8StringPair>> for UserProperties {
    fn from(val: Vec<UTF8StringPair>) -> Self {
        Self { map: val }
    }
}

impl From<Vec<UserProperty>> for UserProperties {
    fn from(val: Vec<UserProperty>) -> Self {
        Self {
            map: val.into_iter().map(UTF8StringPair::from).collect(),
        }
    }
}

impl UserProperties {
    pub(crate) fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map
            .iter()
            .any(|pair| str::from_utf8(&pair.0).unwrap() == key)
    }

    pub fn get<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> {
        self.map
            .iter()
            .filter(move |&pair| str::from_utf8(&pair.0).unwrap() == key)
            .map(|pair| str::from_utf8(&pair.1).unwrap())
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|pair| str::from_utf8(&pair.0).unwrap())
    }

    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|pair| str::from_utf8(&pair.1).unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.map.iter().map(|pair| {
            (
                str::from_utf8(&pair.0).unwrap(),
                str::from_utf8(&pair.1).unwrap(),
            )
        })
    }

    pub(crate) fn push(&mut self, val: UserProperty) {
        self.map.push(UTF8StringPair::from(val));
    }
}

impl fmt::Display for UserProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        self.iter()
            .try_for_each(|(key, val)| write!(f, "\"{}\": \"{}\"", key, val))?;
        write!(f, "}}")
    }
}
