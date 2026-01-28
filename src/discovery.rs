use std::collections::HashSet;

pub struct NodeDiscovery {
    known_nodes: HashSet<String>,
}

impl NodeDiscovery {
    pub fn new() -> Self {
        Self {
            known_nodes: HashSet::new(),
        }
    }

    pub fn with_bootstrap(bootstrap: Vec<String>) -> Self {
        let mut discovery = Self::new();
        for node in bootstrap {
            discovery.add_node(node);
        }
        discovery
    }

    pub fn add_node(&mut self, node: String) {
        self.known_nodes.insert(node);
    }

    pub fn get_neighbors(&self) -> Vec<String> {
        self.known_nodes.iter().cloned().collect()
    }

    pub fn remove_node(&mut self, node: &str) {
        self.known_nodes.remove(node);
    }

    pub fn count(&self) -> usize {
        self.known_nodes.len()
    }
}

impl Default for NodeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
