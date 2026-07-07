//! Analyze noun: compute and inspect workspace structure.
//!
//! Provides analysis commands for Rust workspace organization, dependency graphs,
//! and build ordering — useful for understanding project topology and planning
//! parallel CI/CD pipelines.

use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct AnalyzeCommand;

impl AnalyzeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyzeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for AnalyzeCommand {
    fn name(&self) -> &'static str {
        "analyze"
    }

    fn about(&self) -> &'static str {
        "Analyze workspace structure, dependency graphs, and build order"
    }

    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        #[cfg(feature = "advanced")]
        {
            vec![Box::new(DepOrderVerb)]
        }
        #[cfg(not(feature = "advanced"))]
        {
            vec![]
        }
    }
}

/// `cargo cicd analyze dep-order` — Compute and print the build order of crates.
///
/// This verb uses the workspace dependency graph to determine the order in which
/// crates must be compiled such that all dependencies are available before any
/// crate that consumes them. The output is one crate name per line, in build order,
/// making it suitable for orchestrating parallel CI/CD pipelines.
///
/// If a dependency cycle is detected, the error identifies a crate in the cycle
/// and suggests using `strongly_connected_components()` to find the full cluster
/// that must be broken.
///
/// Only available when cargo-cicd is built with the `advanced` feature flag.
#[cfg(feature = "advanced")]
pub struct DepOrderVerb;

#[cfg(feature = "advanced")]
impl DepOrderVerb {
    fn execute(&self) -> anyhow::Result<()> {
        use crate::advanced::dep_graph::{CycleError, WorkspaceGraph};

        // Parse workspace metadata using cargo metadata.
        let output = std::process::Command::new("cargo")
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("cargo metadata failed");
        }

        let metadata_json = String::from_utf8(output.stdout)?;

        // Parse JSON to extract workspace members and their dependencies.
        let json: serde_json::Value = serde_json::from_str(&metadata_json)?;

        let mut graph = WorkspaceGraph::new();

        // Collect all workspace members.
        if let Some(members) = json["workspace_members"].as_array() {
            for member in members {
                if let Some(member_str) = member.as_str() {
                    // workspace_members entries are like "path/to/crate#0.1.0"
                    // We extract the crate name by looking it up in the packages list.
                    if let Some(package_id) = member_str.split(' ').next() {
                        if let Some(pkg) = json["packages"].as_array().and_then(|pkgs| {
                            pkgs.iter().find(|p| {
                                p["id"]
                                    .as_str()
                                    .map(|id| id.starts_with(package_id))
                                    .unwrap_or(false)
                            })
                        }) {
                            if let Some(crate_name) = pkg["name"].as_str() {
                                graph.add_crate(crate_name);
                            }
                        }
                    }
                }
            }
        }

        // Build the dependency graph by examining resolve information.
        if let Some(resolve) = json["resolve"].as_object() {
            if let Some(nodes) = resolve["nodes"].as_array() {
                for node in nodes {
                    if let Some(node_id) = node["id"].as_str() {
                        if let Some(pkg) = json["packages"].as_array().and_then(|pkgs| {
                            pkgs.iter().find(|p| {
                                p["id"]
                                    .as_str()
                                    .map(|id| id.starts_with(node_id))
                                    .unwrap_or(false)
                            })
                        }) {
                            if let Some(crate_name) = pkg["name"].as_str() {
                                // Add edges for each dependency.
                                if let Some(deps) = node["dependencies"].as_array() {
                                    for dep_id in deps {
                                        if let Some(dep_id_str) = dep_id.as_str() {
                                            if let Some(dep_pkg) =
                                                json["packages"].as_array().and_then(|pkgs| {
                                                    pkgs.iter().find(|p| {
                                                        p["id"]
                                                            .as_str()
                                                            .map(|id| id.starts_with(dep_id_str))
                                                            .unwrap_or(false)
                                                    })
                                                })
                                            {
                                                if let Some(dep_name) = dep_pkg["name"].as_str() {
                                                    graph.add_dependency(crate_name, dep_name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Compute the build order.
        match graph.build_order() {
            Ok(order) => {
                println!("dependency build order");
                println!("======================");
                for (idx, crate_name) in order.iter().enumerate() {
                    println!("{:2}. {}", idx + 1, crate_name);
                }
                Ok(())
            }
            Err(cycle_err) => {
                let CycleError { crate_in_cycle } = &cycle_err;
                eprintln!("error: dependency cycle detected");
                eprintln!("crate involved: {}", crate_in_cycle);
                eprintln!("use 'cargo tree' to inspect the cycle manually");
                anyhow::bail!("{}", cycle_err)
            }
        }
    }
}

#[cfg(feature = "advanced")]
impl VerbCommand for DepOrderVerb {
    fn name(&self) -> &'static str {
        "dep-order"
    }

    fn about(&self) -> &'static str {
        "Compute and print crate build order (dependencies before dependents)"
    }

    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "advanced")]
    #[test]
    fn test_dep_order_builds_graph_and_orders() {
        // Test the DepOrderVerb's logic with a mock workspace structure.
        // Since we parse JSON from `cargo metadata`, we verify the graph building
        // logic independently.

        use crate::advanced::dep_graph::WorkspaceGraph;

        // Simulate a small workspace: app -> lib -> core
        let mut graph = WorkspaceGraph::new();
        graph.add_dependency("app", "lib");
        graph.add_dependency("lib", "core");

        let order = graph
            .build_order()
            .expect("acyclic graph should have a build order");
        assert_eq!(order, vec!["core", "lib", "app"]);
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_dep_order_diamond_dependency() {
        use crate::advanced::dep_graph::WorkspaceGraph;

        // Diamond dependency: root -> (left, right) -> base
        let mut graph = WorkspaceGraph::new();
        graph.add_dependency("root", "left");
        graph.add_dependency("root", "right");
        graph.add_dependency("left", "base");
        graph.add_dependency("right", "base");

        let order = graph.build_order().expect("diamond is acyclic");

        // Verify ordering constraints.
        let pos_base = order.iter().position(|c| c == "base").unwrap();
        let pos_left = order.iter().position(|c| c == "left").unwrap();
        let pos_right = order.iter().position(|c| c == "right").unwrap();
        let pos_root = order.iter().position(|c| c == "root").unwrap();

        assert!(pos_base < pos_left, "base must come before left");
        assert!(pos_base < pos_right, "base must come before right");
        assert!(pos_left < pos_root, "left must come before root");
        assert!(pos_right < pos_root, "right must come before root");
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_dep_order_detects_cycle() {
        use crate::advanced::dep_graph::WorkspaceGraph;

        // Create a cycle: a -> b -> c -> a
        let mut graph = WorkspaceGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");
        graph.add_dependency("c", "a");

        let err = graph
            .build_order()
            .expect_err("cyclic graph should return CycleError");
        assert!(!err.crate_in_cycle.is_empty());
    }
}
