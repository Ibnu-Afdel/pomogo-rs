// Deep Focus duration selection screen.

use chrono::Duration;
use crate::config::Config;
use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::place_center;
use crate::theme::Theme;

pub fn render_duration_picker(
    width: usize,
    height: usize,
    th: &Theme,
    selected_idx: usize,
    cfg: &Config,
) -> String {
    let default_duration = cfg.deep_focus_default_duration_as_duration();
    let color = th.accent();
    let muted = th.muted();

    let rhythm = format!(
        "{} block · {}/{}/{} rhythm every {}",
        format_dur(default_duration),
        format_dur(cfg.deep_focus_work_duration_as_duration()),
        format_dur(cfg.deep_focus_short_break_duration_as_duration()),
        format_dur(cfg.deep_focus_long_break_duration_as_duration()),
        cfg.deep_focus_sessions_before_long_break(),
    );

    let options = [
        "60 min  (1 hour)",
        "120 min (2 hours)",
        "180 min (3 hours)",
        "240 min (4 hours)",
        "Custom duration...",
    ];

    let mut rows = Vec::new();
    rows.push(ansi_bold_fg(color, "Deep Focus Duration"));
    rows.push(ansi_fg(muted, &rhythm));
    rows.push(String::new());

    for (i, opt) in options.iter().enumerate() {
        let mut opt_text = opt.to_string();
        if preset_duration(i) == default_duration {
            opt_text.push_str("  default");
        }

        let indicator = if i == selected_idx { "> " } else { "  " };
        let item = if i == selected_idx {
            ansi_bold_fg(color, &opt_text)
        } else {
            ansi_fg(muted, &opt_text)
        };
        rows.push(format!("{}{}", indicator, item));
    }
    rows.push(String::new());

    rows.push(ansi_fg(
        muted,
        "↓/↑ or tab navigate  ·  1-4 jump  ·  enter select",
    ));

    let content = rows.join("\n");
    let boxed = render_box(&content, color, BorderStyle::Rounded, 4, 1);
    place_center(width, height, &boxed)
}

pub fn preset_duration(idx: usize) -> Duration {
    if idx <= 3 {
        Duration::hours((idx + 1) as i64)
    } else {
        Duration::zero()
    }
}

fn format_dur(d: Duration) -> String {
    let total_mins = d.num_minutes();
    if total_mins % 60 == 0 {
        format!("{}h", total_mins / 60)
    } else {
        format!("{}m", total_mins)
    }
}

