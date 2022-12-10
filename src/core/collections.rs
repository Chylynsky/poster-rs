use crate::core::{base_types::UTF8StringPair, utils::Decoder};
use bytes::Bytes;
use std::str;

use super::properties::UserProperty;

#[derive(Debug, Clone, Default)]
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
            map: val
                .into_iter()
                .map(|property| UTF8StringPair::from(property))
                .collect(),
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
            .find(|&pair| str::from_utf8(&pair.0).unwrap() == key)
            .is_some()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map
            .iter()
            .find(|&pair| str::from_utf8(&pair.0).unwrap() == key)
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
