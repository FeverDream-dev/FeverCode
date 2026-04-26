use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    pub id: String,
    pub file_path: String,
    pub original: String,
    pub replacement: String,
    pub description: String,
    pub approved: Option<bool>,
}

impl PatchProposal {
    pub fn new(
        file_path: impl Into<String>,
        original: impl Into<String>,
        replacement: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
            file_path: file_path.into(),
            original: original.into(),
            replacement: replacement.into(),
            description: description.into(),
            approved: None,
        }
    }

    pub fn full_file(
        file_path: impl Into<String>,
        content: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
            file_path: file_path.into(),
            original: String::new(),
            replacement: content.into(),
            description: description.into(),
            approved: None,
        }
    }

    pub fn render_diff(&self) -> String {
        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", self.file_path));
        diff.push_str(&format!("+++ {} (patch {})\n", self.file_path, self.id));

        let _old_lines: Vec<&str> = self.original.lines().collect();
        let _new_lines: Vec<&str> = self.replacement.lines().collect();

        let chunks = similar::TextDiff::from_lines(&self.original, &self.replacement);

        for change in chunks.iter_all_changes() {
            let sign = match change.tag() {
                similar::ChangeTag::Delete => '-',
                similar::ChangeTag::Insert => '+',
                similar::ChangeTag::Equal => ' ',
            };
            diff.push_str(&format!("{}{}", sign, change));
        }

        diff
    }

    pub fn apply(&self, workspace_root: &Path) -> Result<()> {
        let full_path = workspace_root.join(&self.file_path);

        if self.original.is_empty() && !full_path.exists() {
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &self.replacement)?;
            return Ok(());
        }

        let current = std::fs::read_to_string(&full_path)?;
        if !self.original.is_empty() && !current.contains(&self.original) {
            anyhow::bail!(
                "patch {}: original text not found in {}",
                self.id,
                self.file_path
            );
        }

        let new_content = if self.original.is_empty() {
            self.replacement.clone()
        } else {
            current.replace(&self.original, &self.replacement)
        };

        std::fs::write(&full_path, &new_content)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ApprovalQueue {
    proposals: Vec<PatchProposal>,
}

impl ApprovalQueue {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
        }
    }

    pub fn add(&mut self, proposal: PatchProposal) {
        self.proposals.push(proposal);
    }

    pub fn pending(&self) -> Vec<&PatchProposal> {
        self.proposals
            .iter()
            .filter(|p| p.approved.is_none())
            .collect()
    }

    pub fn approve(&mut self, id: &str) -> Result<()> {
        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("proposal {} not found", id))?;
        proposal.approved = Some(true);
        Ok(())
    }

    pub fn reject(&mut self, id: &str) -> Result<()> {
        let proposal = self
            .proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("proposal {} not found", id))?;
        proposal.approved = Some(false);
        Ok(())
    }

    pub fn apply_approved(&mut self, workspace_root: &Path) -> Result<Vec<String>> {
        let mut applied = Vec::new();
        for proposal in &self.proposals {
            if proposal.approved == Some(true) {
                proposal.apply(workspace_root)?;
                applied.push(proposal.id.clone());
            }
        }
        self.proposals.retain(|p| p.approved != Some(true));
        Ok(applied)
    }

    pub fn render_all(&self) -> String {
        let mut out = String::new();
        for proposal in &self.proposals {
            let status = match proposal.approved {
                Some(true) => "[APPROVED]",
                Some(false) => "[REJECTED]",
                None => "[PENDING]",
            };
            out.push_str(&format!(
                "=== Patch {} {} - {} ===\n{}\n\n",
                proposal.id,
                status,
                proposal.description,
                proposal.render_diff()
            ));
        }
        out
    }

    pub fn clear(&mut self) {
        self.proposals.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn patch_proposal_renders_diff() {
        let p = PatchProposal::new(
            "src/main.rs",
            "fn old() {}",
            "fn new() {}",
            "rename function",
        );
        let diff = p.render_diff();
        assert!(diff.contains("src/main.rs"));
        assert!(diff.contains("fn new() {}"));
    }

    #[test]
    fn patch_proposal_apply_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = PatchProposal::full_file("new_file.txt", "hello world", "create file");
        p.apply(dir.path()).unwrap();
        let content = fs::read_to_string(dir.path().join("new_file.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn patch_proposal_apply_replaces_text() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("code.rs");
        fs::write(&file_path, "fn old() {}").unwrap();

        let p = PatchProposal::new("code.rs", "fn old() {}", "fn new() {}", "rename");
        p.apply(dir.path()).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "fn new() {}");
    }

    #[test]
    fn patch_proposal_rejects_missing_original() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("code.rs");
        fs::write(&file_path, "something else").unwrap();

        let p = PatchProposal::new("code.rs", "fn old() {}", "fn new() {}", "rename");
        assert!(p.apply(dir.path()).is_err());
    }

    #[test]
    fn approval_queue_flow() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "original").unwrap();

        let mut queue = ApprovalQueue::new();
        let p1 = PatchProposal::new("test.txt", "original", "updated", "change text");
        let id = p1.id.clone();
        queue.add(p1);

        assert_eq!(queue.pending().len(), 1);

        queue.approve(&id).unwrap();
        assert_eq!(queue.pending().len(), 0);

        let applied = queue.apply_approved(dir.path()).unwrap();
        assert_eq!(applied.len(), 1);

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "updated");
    }

    #[test]
    fn approval_queue_reject() {
        let mut queue = ApprovalQueue::new();
        let p = PatchProposal::full_file("x.txt", "content", "desc");
        let id = p.id.clone();
        queue.add(p);

        queue.reject(&id).unwrap();
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn approval_queue_clear() {
        let mut queue = ApprovalQueue::new();
        queue.add(PatchProposal::full_file("a.txt", "a", "a"));
        queue.add(PatchProposal::full_file("b.txt", "b", "b"));
        queue.clear();
        assert!(queue.pending().is_empty());
    }
}
