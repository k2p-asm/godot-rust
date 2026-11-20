//! `scene/gui/graph_edit.*`: the connection graph (no drawing).
//!
//! `GraphEdit` owns a list of connections and a per-node index. This port keeps
//! the model exactly:
//!
//! - `Connection` (`from_node`/`to_node`, ports, activity, `keep_alive`);
//! - `connect_node` (idempotent: an existing pair returns `Ok` without adding a
//!   second entry), `is_node_connected`, `get_connection_count`,
//!   `disconnect_node` (first match) and `clear_connections`;
//! - `set_connection_activity` / `reset_all_connection_activity`;
//! - the valid connection-type table (`add`/`remove`/`is_valid`, stored in both
//!   directions like the engine's union key).
//!
//! Node positions, port positions, line geometry, minimap and input handling are
//! not ported; `get_connection_line` is therefore absent.

use std::collections::HashSet;

use crate::string::string_name::StringName;

#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    pub from_node: StringName,
    pub to_node: StringName,
    pub from_port: i32,
    pub to_port: i32,
    pub activity: f32,
    pub keep_alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectResult {
    Ok,
    /// The caller asked to connect a port to itself on the same node+port.
    SelfConnection,
}

#[derive(Clone, Debug, Default)]
pub struct GraphEditData {
    connections: Vec<Connection>,
    valid_types: HashSet<(i32, i32)>,
}

impl GraphEditData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `connect_node`: an already-existing connection is a no-op that still
    /// reports `Ok` (the engine's behaviour).
    pub fn connect_node(
        &mut self,
        from: &str,
        from_port: i32,
        to: &str,
        to_port: i32,
        keep_alive: bool,
    ) -> ConnectResult {
        if self.is_node_connected(from, from_port, to, to_port) {
            return ConnectResult::Ok;
        }
        self.connections.push(Connection {
            from_node: StringName::new(from),
            to_node: StringName::new(to),
            from_port,
            to_port,
            activity: 0.0,
            keep_alive,
        });
        ConnectResult::Ok
    }

    pub fn is_node_connected(&self, from: &str, from_port: i32, to: &str, to_port: i32) -> bool {
        self.connections.iter().any(|connection| {
            connection.from_node.as_string() == from
                && connection.from_port == from_port
                && connection.to_node.as_string() == to
                && connection.to_port == to_port
        })
    }

    /// `get_connection_count`: how many connections touch this node/port.
    pub fn get_connection_count(&self, node: &str, port: i32) -> usize {
        self.connections
            .iter()
            .filter(|connection| {
                (connection.from_node.as_string() == node && connection.from_port == port)
                    || (connection.to_node.as_string() == node && connection.to_port == port)
            })
            .count()
    }

    /// `disconnect_node`: removes the first matching connection.
    pub fn disconnect_node(&mut self, from: &str, from_port: i32, to: &str, to_port: i32) -> bool {
        match self.connections.iter().position(|connection| {
            connection.from_node.as_string() == from
                && connection.from_port == from_port
                && connection.to_node.as_string() == to
                && connection.to_port == to_port
        }) {
            Some(index) => {
                self.connections.remove(index);
                true
            }
            None => false,
        }
    }

    /// `clear_connections`.
    pub fn clear_connections(&mut self) {
        self.connections.clear();
    }

    pub fn get_connections(&self) -> &[Connection] {
        &self.connections
    }

    pub fn get_connection_count_total(&self) -> usize {
        self.connections.len()
    }

    /// `set_connection_activity`: false when the connection is absent.
    pub fn set_connection_activity(
        &mut self,
        from: &str,
        from_port: i32,
        to: &str,
        to_port: i32,
        activity: f32,
    ) -> bool {
        for connection in self.connections.iter_mut() {
            if connection.from_node.as_string() == from
                && connection.from_port == from_port
                && connection.to_node.as_string() == to
                && connection.to_port == to_port
            {
                connection.activity = activity;
                return true;
            }
        }
        false
    }

    pub fn reset_all_connection_activity(&mut self) {
        for connection in self.connections.iter_mut() {
            connection.activity = 0.0;
        }
    }

    /// `add_valid_connection_type`: stores both directions in the union table.
    pub fn add_valid_connection_type(&mut self, from_type: i32, to_type: i32) {
        self.valid_types.insert((from_type, to_type));
        self.valid_types.insert((to_type, from_type));
    }

    pub fn remove_valid_connection_type(&mut self, from_type: i32, to_type: i32) {
        self.valid_types.remove(&(from_type, to_type));
        self.valid_types.remove(&(to_type, from_type));
    }

    /// `is_valid_connection_type`: an empty table accepts every pair (the engine
    /// only rejects pairs explicitly excluded by the table's rules).
    pub fn is_valid_connection_type(&self, from_type: i32, to_type: i32) -> bool {
        if self.valid_types.is_empty() {
            return true;
        }
        self.valid_types.contains(&(from_type, to_type))
    }

    pub fn get_valid_type_count(&self) -> usize {
        self.valid_types.len() / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_and_query() {
        let mut graph = GraphEditData::new();
        assert_eq!(
            graph.connect_node("A", 0, "B", 1, false),
            ConnectResult::Ok
        );
        assert_eq!(graph.get_connection_count_total(), 1);
        assert!(graph.is_node_connected("A", 0, "B", 1));
        assert!(!graph.is_node_connected("B", 1, "A", 0), "direction matters");
        assert!(!graph.is_node_connected("A", 1, "B", 1));

        // Connecting the same pair again is a no-op that reports Ok.
        assert_eq!(
            graph.connect_node("A", 0, "B", 1, false),
            ConnectResult::Ok
        );
        assert_eq!(graph.get_connection_count_total(), 1);

        // A second connection from the same output port is allowed.
        graph.connect_node("A", 0, "C", 2, true);
        assert_eq!(graph.get_connection_count_total(), 2);
        assert_eq!(graph.get_connection_count("A", 0), 2);
        assert_eq!(graph.get_connection_count("B", 1), 1);
        assert_eq!(graph.get_connection_count("C", 2), 1);
        assert_eq!(graph.get_connection_count("C", 0), 0);

        let connections = graph.get_connections();
        assert_eq!(connections[0].from_node.as_string(), "A");
        assert_eq!(connections[1].to_node.as_string(), "C");
        assert!(connections[1].keep_alive);
        assert_eq!(connections[1].activity, 0.0);
    }

    #[test]
    fn disconnect_and_clear() {
        let mut graph = GraphEditData::new();
        graph.connect_node("A", 0, "B", 1, false);
        graph.connect_node("A", 0, "B", 2, false);
        assert!(graph.disconnect_node("A", 0, "B", 1));
        assert!(!graph.disconnect_node("A", 0, "B", 1));
        assert_eq!(graph.get_connection_count_total(), 1);
        assert!(graph.is_node_connected("A", 0, "B", 2));
        graph.clear_connections();
        assert_eq!(graph.get_connection_count_total(), 0);
        assert!(!graph.is_node_connected("A", 0, "B", 2));
    }

    #[test]
    fn activity() {
        let mut graph = GraphEditData::new();
        graph.connect_node("A", 0, "B", 1, false);
        assert!(graph.set_connection_activity("A", 0, "B", 1, 0.75));
        assert_eq!(graph.get_connections()[0].activity, 0.75);
        assert!(graph.set_connection_activity("A", 0, "B", 1, 1.0));
        assert_eq!(graph.get_connections()[0].activity, 1.0);
        assert!(!graph.set_connection_activity("A", 0, "B", 2, 1.0));
        graph.reset_all_connection_activity();
        assert_eq!(graph.get_connections()[0].activity, 0.0);
    }

    #[test]
    fn valid_connection_types() {
        let mut graph = GraphEditData::new();
        // An empty table accepts everything.
        assert!(graph.is_valid_connection_type(1, 2));

        graph.add_valid_connection_type(1, 2);
        assert!(graph.is_valid_connection_type(1, 2));
        assert!(graph.is_valid_connection_type(2, 1), "stored symmetrically");
        assert!(!graph.is_valid_connection_type(1, 3));
        assert_eq!(graph.get_valid_type_count(), 1);

        graph.add_valid_connection_type(3, 4);
        assert_eq!(graph.get_valid_type_count(), 2);
        graph.remove_valid_connection_type(2, 1);
        assert!(!graph.is_valid_connection_type(1, 2));
        assert_eq!(graph.get_valid_type_count(), 1);
        graph.remove_valid_connection_type(9, 9);
        assert_eq!(graph.get_valid_type_count(), 1);
    }

    #[test]
    fn connection_uses_string_names() {
        let mut graph = GraphEditData::new();
        graph.connect_node("Node 1", 0, "Node 2", 0, false);
        assert!(graph.is_node_connected("Node 1", 0, "Node 2", 0));
        // `StringName` interning keeps the names comparable by value.
        assert_eq!(
            graph.get_connections()[0].from_node,
            StringName::new("Node 1")
        );
    }
}
