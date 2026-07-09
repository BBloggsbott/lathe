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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn state_with(value: Value) -> AgentState {
        AgentState(value)
    }

    #[test]
    fn try_from_valid_object_succeeds() {
        let state = AgentState::try_from(json!({"foo": "bar"})).unwrap();
        assert_eq!(state.get("/foo"), Some(&json!("bar")));
    }

    #[test]
    fn try_from_empty_object_errors() {
        let err = AgentState::try_from(json!({})).unwrap_err();
        assert!(matches!(err, AgentStateError::Empty));
    }

    #[test]
    fn try_from_non_object_errors() {
        let err = AgentState::try_from(json!("hello")).unwrap_err();
        assert!(matches!(err, AgentStateError::NotAnObject));
    }

    #[test]
    fn agent_state_error_display() {
        assert_eq!(
            AgentStateError::NotAnObject.to_string(),
            "agent state root must be a JSON object"
        );
        assert_eq!(
            AgentStateError::Empty.to_string(),
            "agent state must not be empty"
        );
    }

    #[test]
    fn default_is_empty() {
        assert!(AgentState::default().is_empty());
    }

    #[test]
    fn get_missing_pointer_returns_none() {
        let state = state_with(json!({"foo": "bar"}));
        assert_eq!(state.get("/missing"), None);
    }

    #[test]
    fn set_creates_intermediate_objects() {
        let mut state = AgentState::default();
        state.set("/a/b/c", json!(42)).unwrap();
        assert_eq!(state.get("/a/b/c"), Some(&json!(42)));
    }

    #[test]
    fn set_overwrites_existing_value() {
        let mut state = state_with(json!({"foo": "bar"}));
        state.set("/foo", json!("baz")).unwrap();
        assert_eq!(state.get("/foo"), Some(&json!("baz")));
    }

    #[test]
    fn set_root_pointer_errors() {
        let mut state = AgentState::default();
        assert!(state.set("/", json!(1)).is_err());
    }

    #[test]
    fn set_empty_pointer_errors() {
        let mut state = AgentState::default();
        assert!(state.set("", json!(1)).is_err());
    }

    #[test]
    fn set_through_non_object_errors() {
        let mut state = state_with(json!({"foo": "bar"}));
        assert!(state.set("/foo/baz", json!(1)).is_err());
    }

    #[test]
    fn set_decodes_json_pointer_escapes() {
        let mut state = AgentState::default();
        state.set("/a~1b", json!("slash")).unwrap();
        state.set("/c~0d", json!("tilde")).unwrap();
        assert_eq!(state.get("/a~1b"), Some(&json!("slash")));
        assert_eq!(state.get("/c~0d"), Some(&json!("tilde")));
    }

    #[test]
    fn exists_reports_presence() {
        let state = state_with(json!({"foo": "bar"}));
        assert!(state.exists("/foo"));
        assert!(!state.exists("/missing"));
    }

    #[test]
    fn select_projects_requested_pointers() {
        let state = state_with(json!({"foo": "bar", "baz": {"qux": 1}, "unused": true}));
        let selected = state
            .select(&vec!["/foo".to_string(), "/baz/qux".to_string()])
            .unwrap();
        assert_eq!(selected.get("/foo"), Some(&json!("bar")));
        assert_eq!(selected.get("/baz/qux"), Some(&json!(1)));
        assert_eq!(selected.get("/unused"), None);
    }

    #[test]
    fn select_missing_pointer_errors() {
        let state = state_with(json!({"foo": "bar"}));
        assert!(state.select(&vec!["/missing".to_string()]).is_err());
    }

    #[test]
    fn is_empty_true_for_empty_object() {
        let state = state_with(json!({}));
        assert!(state.is_empty());
    }

    #[test]
    fn is_empty_false_for_populated_object() {
        let state = state_with(json!({"foo": "bar"}));
        assert!(!state.is_empty());
    }

    #[test]
    fn is_empty_true_for_non_object() {
        let state = state_with(json!("hello"));
        assert!(state.is_empty());
    }

    #[test]
    fn pretty_string_round_trips_as_json() {
        let state = state_with(json!({"foo": "bar"}));
        let pretty = state.pretty_string().unwrap();
        let reparsed: Value = serde_json::from_str(&pretty).unwrap();
        assert_eq!(reparsed, json!({"foo": "bar"}));
    }

    #[test]
    fn into_value_unwraps_underlying_json() {
        let state = state_with(json!({"foo": "bar"}));
        assert_eq!(state.into_value(), json!({"foo": "bar"}));
    }
}
