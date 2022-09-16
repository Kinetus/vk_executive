use serde::ser::SerializeMap;
use std::slice;
use serde_json::value::Value;
use serde::Serialize;

type Pair = (String, Value);

pub struct Params(Vec<Pair>);

impl<const N: usize> From<[Pair; N]> for Params {
    fn from(array: [Pair; N]) -> Self {
        Params(Vec::from(array))
    }
}

impl Params {
    pub fn new() -> Params {
        Params(Vec::new())
    }

    pub fn push<K: ToString>(&mut self, key: K, value: Value) {
        self.0.push((key.to_string(), value))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for Params {
    type Item = Pair;
    type IntoIter = std::vec::IntoIter<Pair>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Params {
    type Item = &'a Pair;
    type IntoIter = slice::Iter<'a, Pair>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Params {
    type Item = &'a mut Pair;
    type IntoIter = slice::IterMut<'a, Pair>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Serialize for Params {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;

        for (k, v) in self.into_iter() {
            map.serialize_entry(k, v)?;
        }
        
        map.end()
    }
}