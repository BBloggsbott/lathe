use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentState(Value);

impl Default for AgentState {
    fn default() -> Self {
        Self(Value::Object(Map::new()))
    }
}

impl AgentState {
    pub fn new(initial: Map<String, Value>) -> Self {
        Self(Value::Object(initial))
    }

    pub fn get(&self, pointer: &str) -> Option<&Value> {
        self.0.pointer(pointer)
    }

    pub fn set(&mut self, pointer: &str, value: Value) -> Result<()> {
        set_value_by_pointer(&mut self.0, pointer, value)
    }

    pub fn exists(&self, pointer: &str) -> bool {
        self.0.pointer(pointer).is_some()
    }

    pub fn select(&self, pointers: &Vec<String>) -> Result<Self> {
        let mut selection = Value::Object(Map::new());

        for pointer in pointers {
            let value_from_state = self.get(pointer.as_str()).ok_or_else(|| {
                anyhow!("Cannot find value for pointer in agent state: {pointer}")
            })?;
            set_value_by_pointer(&mut selection, pointer.as_str(), value_from_state.clone())?;
        }

        Ok(Self(selection))
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_object().is_none() || self.0.as_object().unwrap().is_empty()
    }
}

fn set_value_by_pointer(root: &mut Value, pointer: &str, value: Value) -> Result<()> {
    if pointer == "/" {
        bail!("Cannot update root of state")
    }

    if pointer.is_empty() {
        bail!("Cannot update state with empty pointer")
    }

    let segments: Vec<&str> = pointer.split("/").skip(1).collect();

    let mut current = root;

    for (i, segment) in segments.iter().enumerate() {
        let is_last = i == segment.len() - 1;

        // JSON Pointer Decoding
        let key = segment.replace("~1", "/").replace("~0", "~");

        if is_last {
            match current {
                Value::Object(map) => {
                    map.insert(key, value);
                    return Ok(());
                }
                _ => bail!("Cannot into non object at segment: {key}"),
            }
        } else {
            match current {
                Value::Object(map) => {
                    current = map.entry(key).or_insert_with(|| Value::Object(Map::new()));
                }
                _ => bail!("Cannot into non object at segment: {key}"),
            }
        }
    }

    Ok(())
}
