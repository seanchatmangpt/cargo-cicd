//! Workspace crate dependency graph and build ordering.
//!
//! This module models a cargo workspace as a directed graph where each node
//! is a crate and each edge points from a *dependent* crate to the crate it
//! *depends on*. With that orientation, a topological sort yields a build
//! order in which every dependency is compiled before any crate that needs
//! it — exactly what a build orchestrator wants before scheduling work.
//!
//! Beyond ordering, the graph surfaces cyclic crate clusters (which cargo
//! itself forbids between library crates) and answers reverse-reachability
//! questions such as "if crate X changes, which crates must be rebuilt?".

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use petgraph::algo::{is_cyclic_directed, tarjan_scc, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;

/// Error returned when a build order cannot be produced because the workspace
/// graph contains a dependency cycle.
///
/// Cargo cannot build a set of crates whose dependencies form a cycle, so the
/// orchestrator must surface and break the cycle before proceeding. The error
/// carries the name of one crate participating in the cycle as a starting
/// point for diagnosis; [`WorkspaceGraph::strongly_connected_components`]
/// reports the full cyclic cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleError {
    /// A crate name known to participate in the detected cycle.
    pub crate_in_cycle: String,
}

impl fmt::Display for CycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "dependency cycle detected in workspace graph involving crate `{}`",
            self.crate_in_cycle
        )
    }
}

impl Error for CycleError {}

/// A directed dependency graph over the crates in a cargo workspace.
///
/// Edge direction is *dependent -> dependency*: an edge from `a` to `b` means
/// "crate `a` depends on crate `b`". This orientation makes a plain
/// topological sort emit dependencies before the crates that consume them.
pub struct WorkspaceGraph {
    graph: DiGraph<String, ()>,
    indices: HashMap<String, NodeIndex>,
}

impl WorkspaceGraph {
    /// Create an empty workspace graph.
    pub fn new() -> Self {
        WorkspaceGraph {
            graph: DiGraph::new(),
            indices: HashMap::new(),
        }
    }

    /// Ensure a crate node exists, returning its index. Idempotent.
    fn node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.indices.get(name) {
            return idx;
        }
        let idx = self.graph.add_node(name.to_string());
        self.indices.insert(name.to_string(), idx);
        idx
    }

    /// Register a crate in the workspace. Adding the same crate twice is a
    /// no-op and does not duplicate the node.
    pub fn add_crate(&mut self, name: &str) {
        self.node(name);
    }

    /// Record that `crate_name` depends on `depends_on`.
    ///
    /// Both crates are auto-created if not already present. The resulting edge
    /// runs from the dependent (`crate_name`) to its dependency (`depends_on`).
    pub fn add_dependency(&mut self, crate_name: &str, depends_on: &str) {
        let from = self.node(crate_name);
        let to = self.node(depends_on);
        self.graph.update_edge(from, to, ());
    }

    /// Compute a build order in which every dependency precedes the crates
    /// that depend on it.
    ///
    /// Returns the crate names in build order, or a [`CycleError`] if the
    /// graph contains a dependency cycle.
    pub fn build_order(&self) -> Result<Vec<String>, CycleError> {
        match toposort(&self.graph, None) {
            // `toposort` orders sources (dependents) before targets
            // (dependencies). We want dependencies first, so reverse it.
            Ok(order) => Ok(order
                .into_iter()
                .rev()
                .map(|idx| self.graph[idx].clone())
                .collect()),
            Err(cycle) => Err(CycleError {
                crate_in_cycle: self.graph[cycle.node_id()].clone(),
            }),
        }
    }

    /// Return `true` if the workspace graph contains a dependency cycle.
    // `has_cycle`/`strongly_connected_components`/`dependents_of` are public
    // API completing `WorkspaceGraph` (documented cross-references on
    // `CycleError` above point at them) but have no current call site beyond
    // this module's own unit tests; kept as diagnostics API for future
    // cycle-reporting consumers rather than deleted outright.
    #[allow(dead_code)]
    pub fn has_cycle(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Group crates into their strongly connected components.
    ///
    /// Any component with more than one crate (or a single crate with a
    /// self-edge) is a cyclic cluster that must be broken before the workspace
    /// can be built. Components are returned as lists of crate names.
    #[allow(dead_code)] // see has_cycle note above
    pub fn strongly_connected_components(&self) -> Vec<Vec<String>> {
        tarjan_scc(&self.graph)
            .into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .map(|idx| self.graph[idx].clone())
                    .collect()
            })
            .collect()
    }

    /// Return every crate that transitively depends on `name`.
    ///
    /// These are the crates that must be rebuilt if `name` changes. The set is
    /// computed by following incoming edges (dependent -> dependency) outward
    /// from `name`. The result excludes `name` itself and is unspecified in
    /// order.
    #[allow(dead_code)] // see has_cycle note above
    pub fn dependents_of(&self, name: &str) -> Vec<String> {
        let start = match self.indices.get(name) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        let mut seen: Vec<NodeIndex> = Vec::new();
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            for dependent in self.graph.neighbors_directed(node, Direction::Incoming) {
                if !seen.contains(&dependent) && dependent != start {
                    seen.push(dependent);
                    stack.push(dependent);
                }
            }
        }

        seen.into_iter()
            .map(|idx| self.graph[idx].clone())
            .collect()
    }
}

impl Default for WorkspaceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Position of a crate within a build order, for ordering assertions.
    fn pos(order: &[String], name: &str) -> usize {
        order
            .iter()
            .position(|c| c == name)
            .unwrap_or_else(|| panic!("`{name}` missing from build order: {order:?}"))
    }

    #[test]
    fn linear_chain_build_order() {
        // app -> lib -> core   (app depends on lib, lib depends on core)
        let mut g = WorkspaceGraph::new();
        g.add_dependency("app", "lib");
        g.add_dependency("lib", "core");

        let order = g.build_order().expect("acyclic chain has a build order");
        assert_eq!(order, vec!["core", "lib", "app"]);
        assert!(!g.has_cycle());
    }

    #[test]
    fn diamond_orders_dependency_before_dependents_before_root() {
        // root depends on left and right; both depend on base.
        //        root
        //       /    \
        //    left    right
        //       \    /
        //        base
        let mut g = WorkspaceGraph::new();
        g.add_dependency("root", "left");
        g.add_dependency("root", "right");
        g.add_dependency("left", "base");
        g.add_dependency("right", "base");

        let order = g.build_order().expect("diamond is acyclic");
        let base = pos(&order, "base");
        let left = pos(&order, "left");
        let right = pos(&order, "right");
        let root = pos(&order, "root");

        assert!(base < left, "base must precede left");
        assert!(base < right, "base must precede right");
        assert!(left < root, "left must precede root");
        assert!(right < root, "right must precede root");
    }

    #[test]
    fn cycle_is_detected_and_breaks_build_order() {
        // a -> b -> c -> a
        let mut g = WorkspaceGraph::new();
        g.add_dependency("a", "b");
        g.add_dependency("b", "c");
        g.add_dependency("c", "a");

        assert!(g.has_cycle());
        let err = g
            .build_order()
            .expect_err("cyclic graph has no build order");
        let cyclic = g.strongly_connected_components();
        let cluster = cyclic
            .iter()
            .find(|c| c.len() > 1)
            .expect("a multi-crate component should exist");
        assert!(
            cluster.contains(&err.crate_in_cycle),
            "reported crate {} should belong to the cyclic cluster {:?}",
            err.crate_in_cycle,
            cluster
        );
    }

    #[test]
    fn tarjan_groups_the_cyclic_cluster() {
        // x -> y -> x forms a cluster; standalone is separate.
        let mut g = WorkspaceGraph::new();
        g.add_dependency("x", "y");
        g.add_dependency("y", "x");
        g.add_crate("standalone");

        let components = g.strongly_connected_components();
        let cluster = components
            .iter()
            .find(|c| c.len() > 1)
            .expect("the x/y cycle must form one component");
        assert_eq!(cluster.len(), 2);
        assert!(cluster.contains(&"x".to_string()));
        assert!(cluster.contains(&"y".to_string()));

        assert!(
            components
                .iter()
                .any(|c| c == &vec!["standalone".to_string()]),
            "standalone crate should be its own singleton component"
        );
    }

    #[test]
    fn dependents_of_reports_reverse_reachability() {
        // app -> lib -> core; util -> lib
        let mut g = WorkspaceGraph::new();
        g.add_dependency("app", "lib");
        g.add_dependency("lib", "core");
        g.add_dependency("util", "lib");

        let mut deps = g.dependents_of("core");
        deps.sort();
        // Everything that transitively needs core: lib, app, util.
        assert_eq!(deps, vec!["app", "lib", "util"]);

        // The root crate is depended on by nobody.
        assert!(g.dependents_of("app").is_empty());

        // Unknown crate yields an empty set.
        assert!(g.dependents_of("nonexistent").is_empty());
    }
}
