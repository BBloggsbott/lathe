use lathe_core::executor::Executor;

pub struct ServerState {
    pub executor: Executor,
    pub pipeline_name: String,
}
