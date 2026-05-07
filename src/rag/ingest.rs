use anyhow::Result;
use std::path::Path;

/// Extract text from PDF. Tries `pdftotext` CLI first, falls back to empty string.
pub fn extract_pdf_text(path: &Path) -> Result<String> {
    match std::process::Command::new("pdftotext")
        .arg(path)
        .arg("-")
        .output()
    {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        _ => {
            anyhow::bail!(
                "pdftotext not available or failed for {}. Install poppler-utils.",
                path.display()
            )
        }
    }
}
