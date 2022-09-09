use serde::ser::SerializeMap;
use std::slice;
use serde_json::value::Value;
use serde::Serialize;

pub struct Params(Vec<(String, Value)>);

impl<const N: usize> From<[(String, Value); N]> for Params {
    fn from(array: [(String, Value); N]) -> Self {
        Params(Vec::from(array))
    }
}

impl Params {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for Params {
    type Item = (String, Value);
    type IntoIter = std::vec::IntoIter<(String, Value)>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Params {
    type Item = &'a (String, Value);
    type IntoIter = slice::Iter<'a, (String, Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Params {
    type Item = &'a mut (String, Value);
    type IntoIter = slice::IterMut<'a, (String, Value)>;

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