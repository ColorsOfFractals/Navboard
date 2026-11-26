#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Idea,
    Resource,
    Task,
    Decision,
    Command,
    Variable,
    Condition,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: Uuid,
    pub label: String,
    pub direction: PortDirection,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub kind: NodeKind,
    pub title: String,
    pub content: String,
    pub x: f64,
    pub y: f64,
    pub ports: Vec<Port>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cable {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forge {
    pub id: Uuid,
    pub title: String,
    pub nodes: Vec<Node>,
    pub cables: Vec<Cable>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatchError {
    MissingPort,
    InvalidDirection,
    SameNode,
    DuplicateConnection,
}

impl Forge {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            nodes: Vec::new(),
            cables: Vec::new(),
        }
    }

    pub fn add_node(
        &mut self,
        kind: NodeKind,
        title: impl Into<String>,
        x: f64,
        y: f64,
    ) -> Uuid {
        let id = Uuid::new_v4();

        let ports = vec![
            Port {
                id: Uuid::new_v4(),
                label: "Input".into(),
                direction: PortDirection::Input,
            },
            Port {
                id: Uuid::new_v4(),
                label: "Output".into(),
                direction: PortDirection::Output,
            },
        ];

        self.nodes.push(Node {
            id,
            kind,
            title: title.into(),
            content: String::new(),
            x: if x.is_finite() { x } else { 0.0 },
            y: if y.is_finite() { y } else { 0.0 },
            ports,
        });

        id
    }

    pub fn edit_node(
        &mut self,
        id: Uuid,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.title = title.into();
            node.content = content.into();
            true
        } else {
            false
        }
    }

    pub fn move_node(&mut self, id: Uuid, x: f64, y: f64) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }

        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.x = x;
            node.y = y;
            true
        } else {
            false
        }
    }

    pub fn add_port(
        &mut self,
        node_id: Uuid,
        label: impl Into<String>,
        direction: PortDirection,
    ) -> Option<Uuid> {
        let node = self.nodes.iter_mut().find(|n| n.id == node_id)?;

        let id = Uuid::new_v4();

        node.ports.push(Port {
            id,
            label: label.into(),
            direction,
        });

        Some(id)
    }

    fn locate_port(&self, id: Uuid) -> Option<(Uuid, PortDirection)> {
        self.nodes.iter().find_map(|node| {
            node.ports.iter()
                .find(|port| port.id == id)
                .map(|port| (node.id, port.direction))
        })
    }

    pub fn connect(&mut self, from: Uuid, to: Uuid)
        -> Result<Uuid, PatchError>
    {
        let (from_node, from_direction) =
            self.locate_port(from).ok_or(PatchError::MissingPort)?;

        let (to_node, to_direction) =
            self.locate_port(to).ok_or(PatchError::MissingPort)?;

        if from_direction != PortDirection::Output
            || to_direction != PortDirection::Input
        {
            return Err(PatchError::InvalidDirection);
        }

        if from_node == to_node {
            return Err(PatchError::SameNode);
        }

        if self.cables.iter().any(|c| c.from == from && c.to == to) {
            return Err(PatchError::DuplicateConnection);
        }

        let id = Uuid::new_v4();

        self.cables.push(Cable { id, from, to });

        Ok(id)
    }

    pub fn disconnect(&mut self, id: Uuid) -> bool {
        let before = self.cables.len();
        self.cables.retain(|c| c.id != id);
        self.cables.len() != before
    }

    pub fn remove_node(&mut self, id: Uuid) -> bool {
        let Some(index) = self.nodes.iter().position(|n| n.id == id) else {
            return false;
        };

        let removed = self.nodes.remove(index);

        self.cables.retain(|cable| {
            !removed.ports.iter().any(|port| {
                port.id == cable.from || port.id == cable.to
            })
        });

        true
    }

    pub fn export_relationships(&self) -> String {
        let mut output = format!("# {}\n\n", self.title);

        for node in &self.nodes {
            output.push_str(&format!("- {}\n", node.title));
        }

        output.push_str("\n## Connections\n\n");

        for cable in &self.cables {
            let from = self.nodes.iter().find(|n| {
                n.ports.iter().any(|p| p.id == cable.from)
            });

            let to = self.nodes.iter().find(|n| {
                n.ports.iter().any(|p| p.id == cable.to)
            });

            if let (Some(from), Some(to)) = (from, to) {
                output.push_str(&format!(
                    "- {} -> {}\n",
                    from.title,
                    to.title
                ));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Forge {
        let mut patch = Forge::new("Project");

        patch.add_node(NodeKind::Idea, "Idea", 20.0, 30.0);
        patch.add_node(NodeKind::Task, "Build", 300.0, 30.0);

        patch
    }

    #[test]
    fn create_nodes() {
        let patch = sample();

        assert_eq!(patch.nodes.len(), 2);
        assert_ne!(patch.nodes[0].id, patch.nodes[1].id);
        assert_eq!(patch.nodes[0].ports.len(), 2);
    }

    #[test]
    fn edit_and_move_node() {
        let mut patch = sample();
        let id = patch.nodes[0].id;

        assert!(patch.edit_node(id, "New idea", "Description"));
        assert!(patch.move_node(id, 150.0, 200.0));

        assert_eq!(patch.nodes[0].title, "New idea");
        assert_eq!(patch.nodes[0].x, 150.0);
        assert_eq!(patch.nodes[0].y, 200.0);
    }

    #[test]
    fn connect_modules() {
        let mut patch = sample();

        let from = patch.nodes[0].ports[1].id;
        let to = patch.nodes[1].ports[0].id;

        assert!(patch.connect(from, to).is_ok());
        assert_eq!(patch.cables.len(), 1);
    }

    #[test]
    fn reject_invalid_connections() {
        let mut patch = sample();

        let input = patch.nodes[0].ports[0].id;
        let output = patch.nodes[0].ports[1].id;
        let other_input = patch.nodes[1].ports[0].id;

        assert_eq!(
            patch.connect(input, other_input),
            Err(PatchError::InvalidDirection)
        );

        assert_eq!(
            patch.connect(output, input),
            Err(PatchError::SameNode)
        );

        assert_eq!(
            patch.connect(Uuid::new_v4(), other_input),
            Err(PatchError::MissingPort)
        );
    }

    #[test]
    fn reject_duplicate_cables() {
        let mut patch = sample();

        let from = patch.nodes[0].ports[1].id;
        let to = patch.nodes[1].ports[0].id;

        patch.connect(from, to).unwrap();

        assert_eq!(
            patch.connect(from, to),
            Err(PatchError::DuplicateConnection)
        );
    }

    #[test]
    fn disconnect_cable() {
        let mut patch = sample();

        let from = patch.nodes[0].ports[1].id;
        let to = patch.nodes[1].ports[0].id;

        let cable = patch.connect(from, to).unwrap();

        assert!(patch.disconnect(cable));
        assert!(patch.cables.is_empty());
    }

    #[test]
    fn remove_node_cleans_cables() {
        let mut patch = sample();

        let node = patch.nodes[0].id;
        let from = patch.nodes[0].ports[1].id;
        let to = patch.nodes[1].ports[0].id;

        patch.connect(from, to).unwrap();

        assert!(patch.remove_node(node));
        assert_eq!(patch.nodes.len(), 1);
        assert!(patch.cables.is_empty());
    }

    #[test]
    fn custom_ports() {
        let mut patch = sample();
        let node = patch.nodes[0].id;

        let port = patch.add_port(
            node,
            "Decision",
            PortDirection::Output
        ).unwrap();

        assert_eq!(
            patch.locate_port(port),
            Some((node, PortDirection::Output))
        );
    }

    #[test]
    fn export_relationships() {
        let mut patch = sample();

        let from = patch.nodes[0].ports[1].id;
        let to = patch.nodes[1].ports[0].id;

        patch.connect(from, to).unwrap();

        assert!(patch.export_relationships().contains("Idea -> Build"));
    }

    #[test]
    fn serialization_roundtrip() {
        let mut patch = sample();

        let from = patch.nodes[0].ports[1].id;
        let to = patch.nodes[1].ports[0].id;

        patch.connect(from, to).unwrap();

        let json = serde_json::to_string(&patch).unwrap();
        let restored: Forge = serde_json::from_str(&json).unwrap();

        assert_eq!(patch, restored);
    }
}