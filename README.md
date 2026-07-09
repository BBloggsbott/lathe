# Lathe

> **Early development.** APIs, YAML schema, and behaviour are all subject to change.

A YAML-based AI agent builder. Define agent pipelines as directed graphs in YAML and run them from the CLI.

## How it works

Agents in Lathe are graphs of nodes connected by edges. Each run passes an **`AgentState`** (a JSON object) through the graph in topological order. Nodes read from and write to the state via [JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901) paths (e.g. `/message`, `/output`).

A pipeline YAML describes:
- **nodes** — the processing steps
- **connections** — directed edges between nodes
- **provider_configs** — LLM provider credentials and endpoints

## Workspace structure

```
crates/
  lathe-core/     # Graph, executor, node types, state, templating, YAML serialization
  lathe-cli/      # CLI entry point
  lathe-server/   # Axum HTTP server exposing a pipeline as an API
examples/
  simple_lathe_graph.yaml
```

## Node types

| Type | Description |
|------|-------------|
| `Start` | Entry point for every graph. The initial state is injected here. |
| `LLMNode` | Calls an LLM with a value from state and writes the response back to state. |
| `End` | Terminal node. Declares which state keys are the pipeline's output via `out_pointers`. |

## Supported LLM providers

| Provider | Notes |
|----------|-------|
| `OpenAI` | Reads `OPENAI_API_KEY` from env or the provider config. |
| `LMStudio` | Connects to `http://localhost:1234/v1` by default. |

## Getting started

### Prerequisites

- Rust toolchain (stable)
- An LLM provider (OpenAI API key or a running LM Studio instance)

### Build and install the release binary

```sh
cargo install --path crates/lathe-cli
```

This builds an optimised binary and places it on your `PATH` as `lathe`.

Alternatively, build without installing:

```sh
cargo build --release
# binary at target/release/lathe  (or target/release/lathe.exe on Windows)
```

### Generate an example pipeline

```sh
# installed binary
lathe example simple --provider open-ai --model gpt-5.5

# or from source
cargo run -p lathe-cli -- example simple

# writes examples/simple_lathe_graph.yaml
```

### Run a pipeline

```sh
# installed binary
lathe run --pipeline examples/simple_lathe_graph.yaml --message "Hello!"

# or from source
cargo run -p lathe-cli -- run --pipeline examples/simple_lathe_graph.yaml --message "Hello!"
```

For OpenAI, set your key first:

```sh
export OPENAI_API_KEY=sk-...
# or add it to a .env file
```

### Serve a pipeline over HTTP

```sh
# installed binary
lathe server --pipeline examples/simple_lathe_graph.yaml --host 127.0.0.1 --port 8080

# or from source
cargo run -p lathe-cli -- server --pipeline examples/simple_lathe_graph.yaml
```

This exposes:

| Route | Description |
|-------|-------------|
| `GET /health` | Liveness check; returns `{"status": "ok", "pipeline": "<name>"}`. |
| `POST /invoke` | Runs the pipeline with the request JSON body as the initial `AgentState`, returning the resulting state. |

```sh
curl -X POST http://127.0.0.1:8080/invoke \
  -H 'Content-Type: application/json' \
  -d '{"message": "Hello!"}'
```

## Pipeline YAML format

```yaml
graph_version: V1
name: My Agent

provider_configs:
  my-openai-config:
    id: my-openai-config
    provider: OpenAI
    api_key: null        # falls back to OPENAI_API_KEY env var
    base_url: null       # null = default OpenAI endpoint

nodes:
- !Start
  id: start-node
  label: lathe::nodes::start

- !LLMNode
  id: llm-node
  label: My LLM Step
  provider: OpenAI
  model: gpt-4o-mini
  system_prompt: You are a helpful assistant. The user's name is {{/user_name}}.
  input_key: /message       # JSON Pointer into AgentState
  output_key: /response     # where to write the LLM response
  provider_config_id: my-openai-config

- !End
  id: end-node
  label: lathe::nodes::end
  out_pointers:
  - /response             # keys to surface as output

connections:
- from:
    node_id: start-node
    name: to My LLM Step
  to:
    node_id: llm-node
    name: from lathe::nodes::start
- from:
    node_id: llm-node
    name: to lathe::nodes::end
  to:
    node_id: end-node
    name: from My LLM Step
```

### Templated system prompts

An `LLMNode`'s `system_prompt` can reference values from the `AgentState` using `{{/pointer}}` placeholders, where `pointer` is a [JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901). Placeholders are resolved against the current state just before the node calls the LLM.

### Validation

Loading a graph with validation enabled (the default for `lathe run` and `lathe server`) checks that:
- every connection references a node that exists in the graph
- every leaf node (no outgoing edges) is an `End` node, and vice versa

## Roadmap

- [x] **Web server** — serve a pipeline as an HTTP API endpoint
- [ ] Multi-turn conversation support
- [ ] Additional node types (branching, tool calls, etc.)
- [ ] Cyclic graph support for retry loops and iterative agents
- [ ] **Visual UI** — local graph builder and debugger for authoring and stepping through pipelines
