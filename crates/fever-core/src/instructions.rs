use std::path::Path;

pub fn discover_instructions(workspace: &Path) -> String {
    if !workspace.exists() {
        return String::new();
    }

    let mut found: Vec<String> = Vec::new();
    let mut current = Some(workspace.to_path_buf());

    while let Some(dir) = current {
        let instructions_path = dir.join(".fevercode").join("instructions.md");
        if let Ok(content) = std::fs::read_to_string(&instructions_path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                found.push(trimmed);
            }
        }

        current = dir.parent().map(|p| p.to_path_buf());
        if current.as_ref().is_some_and(|p| *p == dir) {
            break;
        }
    }

    found.reverse();
    found.join("\n\n---\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!("fever_test_{}_{}", name, std::process::id()));
        let _ = fs::create_dir_all(&base);
        base
    }

    fn cleanup_dir(path: &std::path::Path) {
        let _ = fs::remove_dir_all(path);
    }

    fn write_instructions(dir: &std::path::Path, content: &str) {
        let fever_dir = dir.join(".fevercode");
        let _ = fs::create_dir_all(&fever_dir);
        let _ = fs::write(fever_dir.join("instructions.md"), content);
    }

    #[test]
    fn test_no_instructions_file() {
        let tmp = make_temp_dir("no_instr");
        let result = discover_instructions(&tmp);
        assert!(result.is_empty());
        cleanup_dir(&tmp);
    }

    #[test]
    fn test_single_instructions_file() {
        let tmp = make_temp_dir("single_instr");
        write_instructions(&tmp, "Workspace level instructions");
        let result = discover_instructions(&tmp);
        assert_eq!(result, "Workspace level instructions");
        cleanup_dir(&tmp);
    }

    #[test]
    fn test_nested_instructions() {
        let tmp = make_temp_dir("nested_instr");
        write_instructions(&tmp, "Root instructions");

        let child = tmp.join("child");
        let _ = fs::create_dir_all(&child);
        write_instructions(&child, "Child instructions");

        let grandchild = child.join("grandchild");
        let _ = fs::create_dir_all(&grandchild);
        write_instructions(&grandchild, "Grandchild instructions");

        let result = discover_instructions(&grandchild);
        assert!(result.starts_with("Root instructions"));
        assert!(result.contains("Child instructions"));
        assert!(result.contains("Grandchild instructions"));
        assert!(result.contains("---"));
        cleanup_dir(&tmp);
    }

    #[test]
    fn test_missing_workspace() {
        let result = discover_instructions(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_empty());
    }
}
