use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Wrap `text` to `max_width` display columns (word-aware when possible).
pub fn wrap_line(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return Vec::new();
    }
    if text.is_empty() {
        return vec![String::new()];
    }
    if text.width() <= max_width {
        return vec![text.to_owned()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for word in text.split_whitespace() {
        let word_width = word.width();
        if word_width > max_width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current_width = 0;
            }
            lines.extend(hard_wrap(word, max_width));
            continue;
        }

        let needed = if current.is_empty() {
            word_width
        } else {
            current_width + 1 + word_width
        };

        if needed > max_width {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_width = word_width;
        } else if current.is_empty() {
            current.push_str(word);
            current_width = word_width;
        } else {
            current.push(' ');
            current.push_str(word);
            current_width = needed;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Wrap a paragraph that may contain explicit newlines; blank lines are preserved.
pub fn wrap_paragraphs(text: &str, max_width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, para) in text.split("\n\n").enumerate() {
        if idx > 0 {
            out.push(String::new());
        }
        for line in para.lines() {
            if line.is_empty() {
                out.push(String::new());
            } else {
                out.extend(wrap_line(line, max_width));
            }
        }
    }
    out
}

fn hard_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if width + w > max_width && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            width = 0;
        }
        current.push(ch);
        width += w;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_sentence() {
        let lines = wrap_line("Run the full local repository build workflow today", 20);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.width() <= 20));
    }
}
