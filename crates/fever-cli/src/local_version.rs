use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionState {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for VersionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BumpKind {
    Major,
    Minor,
    Patch,
}

impl VersionState {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        VersionState {
            major,
            minor,
            patch,
        }
    }
}

pub struct VersionStore {
    path: PathBuf,
}

impl VersionStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        VersionStore {
            path: path.as_ref().to_path_buf(),
        }
    }
    fn ensure_dir(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
    pub fn load(&self) -> std::io::Result<VersionState> {
        // Ensure directory exists
        self.ensure_dir()?;
        if !self.path.exists() {
            // Initialize with 1.0.0 if missing
            let init = VersionState::new(1, 0, 0);
            self.save(&init)?;
            return Ok(init);
        }
        let mut f = File::open(&self.path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let v: VersionState = serde_json::from_str(&contents)?;
        Ok(v)
    }
    pub fn save(&self, v: &VersionState) -> std::io::Result<()> {
        self.ensure_dir()?;
        let s = serde_json::to_string_pretty(v)?;
        let mut f = File::create(&self.path)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }
    pub fn bump(&self, kind: &BumpKind) -> std::io::Result<VersionState> {
        let mut v = self.load()?;
        match kind {
            BumpKind::Major => {
                v.major = v.major.saturating_add(1);
                v.minor = 0;
                v.patch = 0;
            }
            BumpKind::Minor => {
                v.minor = v.minor.saturating_add(1);
                v.patch = 0;
            }
            BumpKind::Patch => {
                v.patch = v.patch.saturating_add(1);
            }
        }
        self.save(&v)?;
        Ok(v)
    }
}

// Convenience for converting string to BumpKind
pub fn parse_bump(s: &str) -> Option<BumpKind> {
    match s {
        "major" => Some(BumpKind::Major),
        "minor" => Some(BumpKind::Minor),
        "patch" => Some(BumpKind::Patch),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    #[test]
    fn test_load_and_bump_roundtrip() {
        let tmp = env::temp_dir().join("fever_version_test_local");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let store = VersionStore::new(tmp.join("version.json"));
        let v1 = store.load().unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 0);
        assert_eq!(v1.patch, 0);
        let v2 = store.bump(&BumpKind::Patch).unwrap();
        assert_eq!(v2.patch, 1);
        let v3 = store.load().unwrap();
        assert_eq!(v3.patch, 1);
    }
}
