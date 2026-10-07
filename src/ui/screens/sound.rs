// Sound profile selection screen.

use crate::notify::SoundProfile;
use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::place_center;
use crate::theme::Theme;

pub fn render_sound_picker(
    width: usize,
    height: usize,
    th: &Theme,
    selected_idx: usize,
    profiles: &[SoundProfile],
) -> String {
    let color = th.accent();
    let muted = th.muted();
    let txt = th.text();

    let mut rows = Vec::new();
    rows.push(ansi_bold_fg(color, "Sound Profile"));
    rows.push(ansi_fg(muted, "focus start / focus end"));
    rows.push(String::new());

    for (i, p) in profiles.iter().enumerate() {
        let indicator = if i == selected_idx { "> " } else { "  " };
        let line = format!("{:<8}  {} / {}", p.name, p.start_event, p.end_event);
        let name_styled = if i == selected_idx {
            ansi_bold_fg(color, &line)
        } else {
            ansi_fg(txt, &line)
        };

        rows.push(format!("{}{}", indicator, name_styled));
        rows.push(format!("  {}", ansi_fg(muted, p.description)));
    }

    rows.push(String::new());
    rows.push(ansi_fg(
        muted,
        "↓/↑ or tab navigate  ·  space preview  ·  enter select",
    ));

    let content = rows.join("\n");
    let boxed = render_box(&content, color, BorderStyle::Rounded, 4, 1);
    place_center(width, height, &boxed)
}

