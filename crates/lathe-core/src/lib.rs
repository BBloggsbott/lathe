//! Core graph model and execution engine for Lathe pipelines.
//!
//! A pipeline is described by a [`graph::GraphDefinition`] (typically loaded from YAML via
//! [`yaml`]), validated and indexed into a [`graph::LatheGraph`], inflated into runnable
//! [`nodes::LatheNode`]s via [`registry`], and driven end-to-end by an [`executor::Executor`].

pub mod executor;
pub mod graph;
pub mod node_defs;
pub mod nodes;
pub mod provider;
pub mod registry;
pub mod state;
pub mod template;
pub mod yaml;
