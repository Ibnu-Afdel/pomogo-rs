// Key reference for the help overlay. The handlers live in ui/mod.rs; keep
// this table in step with them.

pub struct KeyGroup {
    pub title: &'static str,
    pub keys: &'static [(&'static str, &'static str)],
}

pub const KEY_GROUPS: [KeyGroup; 4] = [
    KeyGroup {
        title: "Session",
        keys: &[
            ("enter", "start, pause or resume"),
            ("n", "skip to the next segment"),
            ("r", "reset"),
            ("d", "plan a deep focus block"),
            ("t", "set the task"),
            ("p", "set the project"),
        ],
    },
    KeyGroup {
        title: "Companion",
        keys: &[
            ("w", "log a glass of water"),
            ("esc", "dismiss a reminder"),
            ("tab", "stats"),
            ("y", "copy stats"),
        ],
    },
    KeyGroup {
        title: "Look",
        keys: &[
            ("T", "next theme"),
            ("L", "next layout"),
            ("e", "next ambient effect"),
            ("S", "zen mode"),
            ("a", "sounds"),
            ("v", "activity label"),
        ],
    },
    KeyGroup {
        title: "General",
        keys: &[("?", "this help"), ("q", "quit (session is saved)")],
    },
];
