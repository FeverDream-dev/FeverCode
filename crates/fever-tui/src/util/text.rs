use unicode_segmentation::UnicodeSegmentation;

pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.graphemes(true).count() <= max_len {
        return s.to_string();
    }
    let truncated: String = s.graphemes(true).take(max_len.saturating_sub(1)).collect();
    format!("{truncated}…")
}

pub fn pad_right(s: &str, width: usize) -> String {
    let grapheme_count = s.graphemes(true).count();
    if grapheme_count >= width {
        return truncate_str(s, width);
    }
    format!(
        "{s:width$}",
        width = width
            - (grapheme_count - s.chars().count()).saturating_sub(s.len() - s.chars().count())
    )
}

pub fn visible_len(s: &str) -> usize {
    s.graphemes(true).count()
}

pub fn center_text(text: &str, width: usize) -> String {
    let len = visible_len(text);
    if len >= width {
        return text.to_string();
    }
    let padding = (width - len) / 2;
    format!("{:padding$}{text}", "")
}

pub fn stepped_divider(width: usize) -> String {
    let pattern = "─ ── ";
    let full = width / pattern.len() + 1;
    let result: String = (0..full).map(|_| pattern).collect();
    result[..width.min(result.len())].to_string()
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if visible_len(line) <= max_width {
            lines.push(line.to_string());
        } else {
            let mut current = String::new();
            for grapheme in line.graphemes(true) {
                if visible_len(&current) + grapheme.len() > max_width {
                    lines.push(current.clone());
                    current.clear();
                }
                current.push_str(grapheme);
            }
            if !current.is_empty() {
                lines.push(current);
            }
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

pub fn ellipsis_count(n: usize, singular: &str) -> String {
    if n == 1 {
        format!("1 {singular}")
    } else {
        format!("{n} {singular}s")
    }
}
