//! Text utilities for formatting and display

use unicode_width::UnicodeWidthStr;

/// Truncate a string to fit within a given width, adding ellipsis if needed
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        return s.to_string();
    }

    let mut width = 0;
    let mut result = String::new();

    for c in s.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + char_width + 1 > max_width {
            result.push('…');
            break;
        }
        result.push(c);
        width += char_width;
    }

    result
}

/// Format a number with thousand separators
pub fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Pad a string to a given width
pub fn pad_right(s: &str, width: usize) -> String {
    let current_width = s.width();
    if current_width >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - current_width))
    }
}

/// Clean up and normalize whitespace in a string
pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
