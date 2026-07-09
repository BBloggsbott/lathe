//! Loading and saving [`GraphDefinition`]s as YAML files.

use crate::graph::{GraphDefinition, LatheGraph};
use anyhow::Result;
use std::fs::File;
use std::path::Path;

/// Writes `graph`'s underlying [`GraphDefinition`] to `path` as YAML.
pub fn save(graph: &LatheGraph, path: &Path) -> Result<()> {
    let out_file = File::create(path)?;
    serde_yaml::to_writer(out_file, &graph.definition)?;

    Ok(())
}

/// Reads a [`GraphDefinition`] from the YAML file at `path` and builds a [`LatheGraph`] from
/// it, optionally validating it (see [`LatheGraph::from_def`]).
pub fn load(path: &Path, validate: bool) -> Result<LatheGraph> {
    let file = File::open(path)?;
    let graph_definition: GraphDefinition = serde_yaml::from_reader(file)?;
    let graph = LatheGraph::from_def(graph_definition, validate)?;
    Ok(graph)
}
