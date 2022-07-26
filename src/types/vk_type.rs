use std::collections::HashMap;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeSeq};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum Value {
    String(String),
    Text(String),
    Integer(i32),
    Positive(u32),
    CheckBox(bool),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Value::String(ref string) => {
                serializer.serialize_str(string)
            }
            Value::Text(ref text) => {
                serializer.serialize_str(text)
            }
            Value::Integer(ref number) => {
                serializer.serialize_i32(*number)
            }
            Value::Positive(ref number) => {
                serializer.serialize_u32(*number)
            }
            Value::CheckBox(ref bool) => {
                serializer.serialize_bool(*bool)
            }
            Value::Object(ref hashmap) => {
                let mut map = serializer.serialize_map(Some(hashmap.len()))?;
                
                for (k, v) in hashmap {
                    map.serialize_entry(k, v)?;
                }

                map.end()
            }
            Value::Array(ref array) => {
                let mut seq = serializer.serialize_seq(Some(array.len()))?;

                for element in array {
                    seq.serialize_element(element)?;
                }

                seq.end()
            }
        }
    }
}