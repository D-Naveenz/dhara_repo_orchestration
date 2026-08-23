use std::time::{Duration, Instant};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Delay between marquee column steps.
pub const MARQUEE_STEP: Duration = Duration::from_millis(120);
/// Hold at each end before reversing direction.
pub const MARQUEE_PAUSE: Duration = Duration::from_millis(800);

/// Ping-pong horizontal scroll for overflowing single-line labels.
#[derive(Debug, Clone)]
pub struct LabelMarquee {
    path_key: String,
    offset: usize,
    forward: bool,
    next_step: Instant,
    pause_until: Instant,
}

impl Default for LabelMarquee {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelMarquee {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            path_key: String::new(),
            offset: 0,
            forward: true,
            next_step: now,
            pause_until: now + MARQUEE_PAUSE,
        }
    }

    fn reset(&mut self, path_key: &str, now: Instant) {
        self.path_key = path_key.to_owned();
        self.offset = 0;
        self.forward = true;
        self.next_step = now;
        self.pause_until = now + MARQUEE_PAUSE;
    }

    /// Advance ping-pong state; returns the column offset to paint.
    pub fn advance(
        &mut self,
        path_key: &str,
        label: &str,
        max_width: usize,
        now: Instant,
    ) -> usize {
        let overflow = label.width().saturating_sub(max_width);
        if overflow == 0 {
            self.reset(path_key, now);
            return 0;
        }

        if self.path_key != path_key {
            self.reset(path_key, now);
            return 0;
        }

        self.offset = self.offset.min(overflow);

        if now < self.pause_until {
            return self.offset;
        }

        if now < self.next_step {
            return self.offset;
        }

        self.next_step = now + MARQUEE_STEP;

        if self.forward {
            if self.offset >= overflow {
                self.forward = false;
                self.pause_until = now + MARQUEE_PAUSE;
            } else {
                self.offset += 1;
                if self.offset >= overflow {
                    self.forward = false;
                    self.pause_until = now + MARQUEE_PAUSE;
                }
            }
        } else if self.offset == 0 {
            self.forward = true;
            self.pause_until = now + MARQUEE_PAUSE;
        } else {
            self.offset -= 1;
            if self.offset == 0 {
                self.forward = true;
                self.pause_until = now + MARQUEE_PAUSE;
            }
        }

        self.offset
    }
}

/// Truncate on the right to `max_width` display columns, appending `…` when clipped.
pub fn clip_right(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let full_width = text.width();
    if full_width <= max_width {
        return text.to_owned();
    }

    let ellipsis = '…';
    let ellipsis_width = ellipsis.width().unwrap_or(1);
    if max_width <= ellipsis_width {
        return ellipsis.to_string();
    }

    let budget = max_width - ellipsis_width;
    let mut out = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if width + w > budget {
            break;
        }
        out.push(ch);
        width += w;
    }
    out.push(ellipsis);
    out
}

/// Skip `offset` display columns, then take up to `max_width` (no ellipsis).
pub fn clip_window(text: &str, offset: usize, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut skipped = 0usize;
    let mut started = offset == 0;
    let mut out = String::new();
    let mut width = 0usize;

    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if !started {
            if skipped + w > offset {
                skipped += w;
                started = true;
                continue;
            }
            skipped += w;
            if skipped == offset {
                started = true;
            }
            continue;
        }

        if width + w > max_width {
            break;
        }
        out.push(ch);
        width += w;
    }

    out
}

/// Left padding (display columns) to center `text` in a slot of `slot_width`.
pub fn center_offset(text: &str, slot_width: usize) -> usize {
    if text.width() >= slot_width {
        0
    } else {
        slot_width.saturating_sub(text.width()) / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_window_skips_offset_and_limits_width() {
        assert_eq!(clip_window("abcdefghij", 0, 4), "abcd");
        assert_eq!(clip_window("abcdefghij", 3, 4), "defg");
        assert_eq!(clip_window("abcdefghij", 8, 4), "ij");
        assert_eq!(clip_window("short", 0, 20), "short");
    }

    #[test]
    fn marquee_resets_on_path_change_and_steps_forward() {
        let mut m = LabelMarquee::new();
        let t0 = Instant::now();
        assert_eq!(m.advance("a", "abcdefghijklmnopqrstuvwxyz", 5, t0), 0);

        let t1 = t0 + MARQUEE_PAUSE + MARQUEE_STEP;
        assert_eq!(m.advance("a", "abcdefghijklmnopqrstuvwxyz", 5, t1), 1);

        let t2 = t1 + MARQUEE_STEP;
        assert_eq!(m.advance("b", "abcdefghijklmnopqrstuvwxyz", 5, t2), 0);
    }

    #[test]
    fn center_offset_short_text() {
        assert_eq!(center_offset("Hi", 10), 4);
        assert_eq!(center_offset("Hello", 5), 0);
    }
}
