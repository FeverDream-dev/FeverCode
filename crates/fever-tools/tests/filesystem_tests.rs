use fever_core::{ExecutionContext, Tool};
use fever_tools::FilesystemTool;
use serde_json::json;

fn make_temp_dir() -> std::path::PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("fever_test_{id}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
}

fn make_ctx() -> ExecutionContext {
    ExecutionContext::new("test".to_string(), "test".to_string())
}

#[tokio::test]
async fn test_filesystem_tool_name_and_description() {
    let tool = FilesystemTool::new();
    assert_eq!(tool.name(), "filesystem");
    assert!(!tool.description().is_empty());
}

#[tokio::test]
async fn test_filesystem_tool_schema() {
    let tool = FilesystemTool::new();
    let schema = tool.schema();
    assert_eq!(schema.name, "filesystem");
    assert!(!schema.description.is_empty());
    // Parameters should be valid JSON with type "object"
    assert_eq!(schema.parameters["type"], "object");
    assert!(schema.parameters["properties"].is_object());
}

#[tokio::test]
async fn test_filesystem_read_existing_file() {
    let dir = make_temp_dir();
    let file_path = dir.join("hello.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "read", "path": file_path.to_str().unwrap()}), &make_ctx())
        .await
        .unwrap();

    assert_eq!(result["content"], "hello world");
    assert_eq!(result["path"], file_path.to_str().unwrap());
    cleanup(&dir);
}

#[tokio::test]
async fn test_filesystem_read_nonexistent_file() {
    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "read", "path": "/tmp/fever_nonexistent_file_99999.txt"}), &make_ctx())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_write_and_read() {
    let dir = make_temp_dir();
    let file_path = dir.join("write_test.txt");
    let content = "written by fever test";

    let tool = FilesystemTool::new();

    // Write
    let write_result = tool
        .execute(json!({"action": "write", "path": file_path.to_str().unwrap(), "content": content}), &make_ctx())
        .await
        .unwrap();
    assert_eq!(write_result["success"], true);
    assert_eq!(write_result["bytes_written"], content.len());

    // Read back
    let read_result = tool
        .execute(json!({"action": "read", "path": file_path.to_str().unwrap()}), &make_ctx())
        .await
        .unwrap();
    assert_eq!(read_result["content"], content);

    cleanup(&dir);
}

#[tokio::test]
async fn test_filesystem_write_missing_path() {
    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "write", "content": "some content"}), &make_ctx())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_write_missing_content() {
    let dir = make_temp_dir();
    let file_path = dir.join("missing_content.txt");

    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "write", "path": file_path.to_str().unwrap()}), &make_ctx())
        .await;

    assert!(result.is_err());
    cleanup(&dir);
}

#[tokio::test]
async fn test_filesystem_list_directory() {
    let dir = make_temp_dir();

    // Create files and subdirectories
    std::fs::write(dir.join("file1.txt"), "a").unwrap();
    std::fs::write(dir.join("file2.rs"), "b").unwrap();
    std::fs::create_dir(dir.join("subdir1")).unwrap();
    std::fs::create_dir(dir.join("subdir2")).unwrap();

    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "list", "path": dir.to_str().unwrap()}), &make_ctx())
        .await
        .unwrap();

    let files: Vec<&str> = result["files"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    let dirs: Vec<&str> = result["directories"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();

    assert_eq!(files.len(), 2);
    assert!(files.contains(&"file1.txt"));
    assert!(files.contains(&"file2.rs"));
    assert_eq!(dirs.len(), 2);
    assert!(dirs.contains(&"subdir1"));
    assert!(dirs.contains(&"subdir2"));

    cleanup(&dir);
}

#[tokio::test]
async fn test_filesystem_list_nonexistent_directory() {
    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "list", "path": "/tmp/fever_nonexistent_dir_99999"}), &make_ctx())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_exists_true() {
    let dir = make_temp_dir();
    let file_path = dir.join("exists_test.txt");
    std::fs::write(&file_path, "data").unwrap();

    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "exists", "path": file_path.to_str().unwrap()}), &make_ctx())
        .await
        .unwrap();

    assert_eq!(result["exists"], true);
    cleanup(&dir);
}

#[tokio::test]
async fn test_filesystem_exists_false() {
    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "exists", "path": "/tmp/fever_nonexistent_99999.txt"}), &make_ctx())
        .await
        .unwrap();

    assert_eq!(result["exists"], false);
}

#[tokio::test]
async fn test_filesystem_delete_file() {
    let dir = make_temp_dir();
    let file_path = dir.join("to_delete.txt");
    std::fs::write(&file_path, "delete me").unwrap();
    assert!(file_path.exists());

    let tool = FilesystemTool::new();

    // Delete
    let result = tool
        .execute(json!({"action": "delete", "path": file_path.to_str().unwrap()}), &make_ctx())
        .await
        .unwrap();
    assert_eq!(result["success"], true);

    // Verify gone
    assert!(!file_path.exists());
    cleanup(&dir);
}

#[tokio::test]
async fn test_filesystem_delete_nonexistent() {
    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "delete", "path": "/tmp/fever_nonexistent_99999.txt"}), &make_ctx())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_unknown_action() {
    let tool = FilesystemTool::new();
    let result = tool
        .execute(json!({"action": "explode", "path": "/tmp/test"}), &make_ctx())
        .await;

    assert!(result.is_err());
}
