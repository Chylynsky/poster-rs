use crate::core::{base_types::UTF8StringPair, properties::UserProperty};
use std::{fmt, str};

/// Map collection for reading user properties as key-value pairs from packets.
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

    /// Returns a number of key-value pairs in the container.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the container is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns `true` if the container contains `at least one` element with the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.map
            .iter()
            .any(|pair| str::from_utf8(&pair.0).unwrap() == key)
    }

    /// Returns an iterator to the values under the given key.
    pub fn get<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> {
        self.map
            .iter()
            .filter(move |&pair| str::from_utf8(&pair.0).unwrap() == key)
            .map(|pair| str::from_utf8(&pair.1).unwrap())
    }

    /// Returns an iterator which iterates over the keys. Note that it can contain duplicates.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|pair| str::from_utf8(&pair.0).unwrap())
    }

    /// Returns an iterator which iterates over the keys.
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|pair| str::from_utf8(&pair.1).unwrap())
    }

    /// Returns an iterator which iterates over key-value tuples.
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

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn size() {
        let properties = UserProperties::new();
        assert!(properties.is_empty());
        assert_eq!(properties.len(), 0);
    }

    #[test]
    fn access() {
        let mut properties = UserProperties::new();
        properties.push(UserProperty::from(UTF8StringPair(
            Bytes::from_static("key0".as_bytes()),
            Bytes::from_static("val0".as_bytes()),
        )));
        properties.push(UserProperty::from(UTF8StringPair(
            Bytes::from_static("key1".as_bytes()),
            Bytes::from_static("val1".as_bytes()),
        )));
        properties.push(UserProperty::from(UTF8StringPair(
            Bytes::from_static("key1".as_bytes()),
            Bytes::from_static("val2".as_bytes()),
        )));

        assert!(!properties.is_empty());
        assert_eq!(properties.len(), 3);

        assert!(properties.contains_key("key0"));
        assert!(properties.contains_key("key1"));
        assert_eq!(properties.get("key0").collect::<Vec<&str>>(), ["val0"]);
        assert_eq!(
            properties.get("key1").collect::<Vec<&str>>(),
            ["val1", "val2"]
        );
        assert_eq!(
            properties.keys().collect::<Vec<&str>>(),
            ["key0", "key1", "key1"]
        );
        assert_eq!(
            properties.values().collect::<Vec<&str>>(),
            ["val0", "val1", "val2"]
        );
        assert_eq!(
            properties.iter().collect::<Vec<(&str, &str)>>(),
            [("key0", "val0"), ("key1", "val1"), ("key1", "val2")]
        );
    }

    // #[test]
    // fn display() {
    //     let mut properties = UserProperties::new();
    //     assert_eq!(format!("{}", properties), "{}");

    //     properties.push(UserProperty::from(UTF8StringPair(
    //         Bytes::from_static("key0".as_bytes()),
    //         Bytes::from_static("val0".as_bytes()),
    //     )));

    //     assert_eq!(format!("{}", properties), "{\"key0\": \"val0\"}");

    //     properties.push(UserProperty::from(UTF8StringPair(
    //         Bytes::from_static("key1".as_bytes()),
    //         Bytes::from_static("val1".as_bytes()),
    //     )));

    //     assert_eq!(
    //         format!("{}", properties),
    //         "{\"key0\": \"val0\", \"key1\": \"val1\"}"
    //     );
    // }
}
