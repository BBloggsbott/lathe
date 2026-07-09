use lathe_core::executor::Executor;

/// Shared Axum application state: the pipeline executor and its display name, wrapped in
/// [`std::sync::Arc`] by [`crate::app`] for cheap cloning across handlers.
pub struct ServerState {
    pub executor: Executor,
    pub pipeline_name: String,
}
