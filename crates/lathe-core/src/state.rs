use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Errors from converting a raw JSON [`Value`] into an [`AgentState`] via [`TryFrom`].
#[derive(Debug)]
pub enum AgentStateError {
    NotAnObject,
    Empty,
}

impl std::fmt::Display for AgentStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAnObject => write!(f, "agent state root must be a JSON object"),
            Self::Empty => write!(f, "agent state must not be empty"),
        }
    }
}

/// The mutable JSON document threaded through a pipeline's nodes. Values are addressed by
/// [RFC 6901 JSON Pointer](https://www.rfc-editor.org/rfc/rfc6901) strings (e.g. `/foo/bar`);
/// the root must always be a JSON object.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentState(Value);

impl Default for AgentState {
    fn default() -> Self {
        Self(Value::Object(Map::new()))
    }
}

impl TryFrom<Value> for AgentState {
    type Error = AgentStateError;

    /// Wraps `value` as an [`AgentState`]; errors if it isn't a non-empty JSON object.
    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) if map.is_empty() => Err(AgentStateError::Empty),
            Value::Object(map) => Ok(AgentState::new(map)),
            _ => Err(AgentStateError::NotAnObject),
        }
    }
}

impl AgentState {
    /// Creates a new state wrapping the given JSON object as its root.
    pub fn new(initial: Map<String, Value>) -> Self {
        Self(Value::Object(initial))
    }

    /// Looks up a value by JSON Pointer (e.g. `/foo/bar`), returning `None` if it doesn't exist.
    pub fn get(&self, pointer: &str) -> Option<&Value> {
        self.0.pointer(pointer)
    }

    /// Sets the value at the given JSON Pointer, creating intermediate objects as needed.
    /// Errors on the root pointer, an empty pointer, or a path that traverses a non-object.
    pub fn set(&mut self, pointer: &str, value: Value) -> Result<()> {
        set_value_by_pointer(&mut self.0, pointer, value)
    }

    /// Returns `true` if a value exists at the given JSON Pointer.
    pub fn exists(&self, pointer: &str) -> bool {
        self.0.pointer(pointer).is_some()
    }

    /// Builds a new state containing only the values at `pointers`, preserving their original
    /// paths. Used by [`crate::nodes::end::EndNode`] to project the final output. Errors if any
    /// pointer is missing from this state.
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

    /// `true` if the root is not an object, or is an empty object.
    pub fn is_empty(&self) -> bool {
        self.0.as_object().is_none() || self.0.as_object().unwrap().is_empty()
    }

    /// Pretty-prints the state as JSON, e.g. for CLI output.
    pub fn pretty_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.0)
    }

    /// Unwraps the state into its underlying JSON [`Value`].
    pub fn into_value(self) -> Value {
        self.0
    }
}

/// Inserts `value` into `root` at the given JSON Pointer, creating any missing intermediate
/// objects along the way. Errors on the root pointer (`/`), an empty pointer, or if any segment
/// of the path traverses a non-object value.
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
        let is_last = i == segments.len() - 1;

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
