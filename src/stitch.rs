#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub id: Uuid,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stitch {
    pub id: Uuid,
    pub title: String,
    pub steps: Vec<Step>,
}

impl Stitch {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            steps: Vec::new(),
        }
    }

    pub fn add(&mut self, text: impl Into<String>) -> Option<Uuid> {
        let text = text.into();

        if text.trim().is_empty() {
            return None;
        }

        let id = Uuid::new_v4();

        self.steps.push(Step { id, text });

        Some(id)
    }

    pub fn edit(&mut self, id: Uuid, text: impl Into<String>) -> bool {
        let text = text.into();

        if text.trim().is_empty() {
            return false;
        }

        if let Some(step) = self.steps.iter_mut().find(|s| s.id == id) {
            step.text = text;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: Uuid) -> bool {
        let before = self.steps.len();
        self.steps.retain(|step| step.id != id);
        self.steps.len() != before
    }

    pub fn duplicate(&mut self, id: Uuid) -> Option<Uuid> {
        let index = self.steps.iter().position(|s| s.id == id)?;

        let mut copy = self.steps[index].clone();
        copy.id = Uuid::new_v4();

        let new_id = copy.id;

        self.steps.insert(index + 1, copy);

        Some(new_id)
    }

    pub fn move_to(&mut self, id: Uuid, destination: usize) -> bool {
        if destination >= self.steps.len() {
            return false;
        }

        let Some(index) = self.steps.iter().position(|s| s.id == id) else {
            return false;
        };

        let step = self.steps.remove(index);
        self.steps.insert(destination, step);

        true
    }

    pub fn export_markdown(&self) -> String {
        let mut output = format!("# {}\n\n", self.title);

        for (index, step) in self.steps.iter().enumerate() {
            let mut lines = step.text.lines();
            let first = lines.next().unwrap_or_default();

            output.push_str(&format!("{}. {}\n", index + 1, first));

            for line in lines {
                output.push_str(&format!("    {}\n", line));
            }
        }

        output
    }

    pub fn export_checklist(&self) -> String {
        let mut output = format!("# {}\n\n", self.title);

        for step in &self.steps {
            let mut lines = step.text.lines();

            output.push_str(&format!(
                "- [ ] {}\n",
                lines.next().unwrap_or_default()
            ));

            for line in lines {
                output.push_str(&format!("    {}\n", line));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_steps() {
        let mut process = Stitch::new("Troubleshooting");

        assert!(process.add("Identify issue").is_some());
        assert!(process.add("Verify resolution").is_some());
        assert_eq!(process.steps.len(), 2);
    }

    #[test]
    fn reject_empty_steps() {
        let mut process = Stitch::new("Test");

        assert!(process.add("   ").is_none());
        assert!(process.steps.is_empty());
    }

    #[test]
    fn edit_step() {
        let mut process = Stitch::new("Test");
        let id = process.add("Original").unwrap();

        assert!(process.edit(id, "Updated"));
        assert_eq!(process.steps[0].text, "Updated");
        assert_eq!(process.steps[0].id, id);
    }

    #[test]
    fn reorder_steps() {
        let mut process = Stitch::new("Test");

        let first = process.add("First").unwrap();
        process.add("Second");
        process.add("Third");

        assert!(process.move_to(first, 2));
        assert_eq!(process.steps[2].id, first);
        assert_eq!(process.steps[0].text, "Second");
    }

    #[test]
    fn duplicate_step() {
        let mut process = Stitch::new("Test");
        let original = process.add("Inspect logs").unwrap();

        let copy = process.duplicate(original).unwrap();

        assert_ne!(original, copy);
        assert_eq!(process.steps[0].text, process.steps[1].text);

        process.edit(copy, "Inspect other logs");

        assert_eq!(process.steps[0].text, "Inspect logs");
    }

    #[test]
    fn remove_step() {
        let mut process = Stitch::new("Test");
        let id = process.add("Temporary").unwrap();

        assert!(process.remove(id));
        assert!(!process.remove(id));
        assert!(process.steps.is_empty());
    }

    #[test]
    fn export_preserves_order() {
        let mut process = Stitch::new("Procedure");

        let first = process.add("First").unwrap();
        process.add("Second");

        process.move_to(first, 1);

        assert_eq!(
            process.export_markdown(),
            "# Procedure\n\n1. Second\n2. First\n"
        );

        assert_eq!(
            process.export_checklist(),
            "# Procedure\n\n- [ ] Second\n- [ ] First\n"
        );
    }

    #[test]
    fn serialization_roundtrip() {
        let mut process = Stitch::new("Test");
        process.add("Persistent step");

        let json = serde_json::to_string(&process).unwrap();
        let restored: Stitch = serde_json::from_str(&json).unwrap();

        assert_eq!(process, restored);
    }
}