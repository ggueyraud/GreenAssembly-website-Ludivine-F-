use serde::{de::Deserializer, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, std::cmp::PartialEq, Serialize)]
pub enum Patch<T> {
    Undefined,
    Null,
    Value(T),
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Patch::Undefined
    }
}

impl<T> From<Option<T>> for Patch<T> {
    fn from(opt: Option<T>) -> Patch<T> {
        match opt {
            Some(v) => Patch::Value(v),
            None => Patch::Null,
        }
    }
}

impl<'de, T> Deserialize<'de> for Patch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}

pub fn extract_fields(form: &impl Serialize) -> std::collections::HashMap<String, Value> {
    let mut need_updated_fields = HashMap::<String, Value>::new();

    for (name, obj) in serde_json::json!(form).as_object().unwrap().iter() {
        match obj {
            Value::String(_) => {
                if obj.as_str().unwrap() != "Undefined" {
                    need_updated_fields.insert(name.to_string(), obj.clone());
                }
            }
            Value::Object(map) => {
                if let Some(value) = map.get("Value") {
                    need_updated_fields.insert(name.to_string(), value.clone());
                }
            }
            _ => {}
        }
    }

    need_updated_fields
}
