use std::collections::HashMap;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeSeq};
use std::string::String as StdString;

pub type String = StdString;
pub type Integer = i64;
pub type Positive = u64;
pub type Boolean = bool;
pub type Object = HashMap<StdString, Value>;
pub type Array = Vec<Value>;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum Value {
    String(String),
    Integer(Integer),
    Positive(Positive),
    Boolean(Boolean),
    Object(Object),
    Array(Array),
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
            Value::Integer(ref number) => {
                serializer.serialize_i64(*number)
            }
            Value::Positive(ref number) => {
                serializer.serialize_u64(*number)
            }
            Value::Boolean(ref bool) => {
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
