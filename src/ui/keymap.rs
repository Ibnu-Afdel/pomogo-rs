// Key bindings for the PomoGo TUI.

use crate::ui::screens::HelpBinding;

#[derive(Debug, Clone)]
pub struct KeyBindingDef {
    pub keys: &'static [&'static str],
    pub description: &'static str,
}

impl KeyBindingDef {
    pub fn matches(&self, key_str: &str) -> bool {
        for k in self.keys {
            if *k == key_str {
                return true;
            }
            if *k == "space" && key_str == " " {
                return true;
            }
        }
        false
    }
}

pub struct KeyMap {
    pub start: KeyBindingDef,
    pub pause_resume: KeyBindingDef,
    pub skip: KeyBindingDef,
    pub task: KeyBindingDef,
    pub project: KeyBindingDef,
    pub deep_focus: KeyBindingDef,
    pub toggle_stats: KeyBindingDef,
    pub copy_stats: KeyBindingDef,
    pub reset: KeyBindingDef,
    pub cycle_theme: KeyBindingDef,
    pub cycle_layout: KeyBindingDef,
    pub sound_picker: KeyBindingDef,
    pub toggle_zen: KeyBindingDef,
    pub cycle_effects: KeyBindingDef,
    pub cycle_verb: KeyBindingDef,
    pub back: KeyBindingDef,
    pub quit: KeyBindingDef,
    pub help: KeyBindingDef,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            start: KeyBindingDef { keys: &["s"], description: "Start the focus session" },
            pause_resume: KeyBindingDef { keys: &["space"], description: "Pause / resume" },
            skip: KeyBindingDef { keys: &["n"], description: "Skip to next phase" },
            task: KeyBindingDef { keys: &["t"], description: "Set current task" },
            project: KeyBindingDef { keys: &["p"], description: "Set current project" },
            deep_focus: KeyBindingDef { keys: &["d"], description: "Choose Deep Focus duration" },
            toggle_stats: KeyBindingDef { keys: &["tab"], description: "Toggle statistics view" },
            copy_stats: KeyBindingDef { keys: &["y"], description: "Copy stats to clipboard" },
            reset: KeyBindingDef { keys: &["r"], description: "Reset and clear state" },
            cycle_theme: KeyBindingDef { keys: &["T"], description: "Cycle theme" },
            cycle_layout: KeyBindingDef { keys: &["L"], description: "Cycle layout" },
            sound_picker: KeyBindingDef { keys: &["a"], description: "Choose sound profile" },
            toggle_zen: KeyBindingDef { keys: &["S"], description: "Toggle screenshot mode" },
            cycle_effects: KeyBindingDef { keys: &["e"], description: "Cycle ambient effects" },
            cycle_verb: KeyBindingDef { keys: &["v"], description: "Cycle activity verb" },
            back: KeyBindingDef { keys: &["esc"], description: "Back / close overlay" },
            quit: KeyBindingDef { keys: &["q", "ctrl+c"], description: "Quit" },
            help: KeyBindingDef { keys: &["?"], description: "Toggle help overlay" },
        }
    }
}

impl KeyMap {
    pub fn help_bindings(&self) -> Vec<HelpBinding> {
        vec![
            HelpBinding { keys: "s", description: self.start.description },
            HelpBinding { keys: "space", description: self.pause_resume.description },
            HelpBinding { keys: "n", description: self.skip.description },
            HelpBinding { keys: "t", description: self.task.description },
            HelpBinding { keys: "p", description: self.project.description },
            HelpBinding { keys: "d", description: self.deep_focus.description },
            HelpBinding { keys: "Tab", description: self.toggle_stats.description },
            HelpBinding { keys: "y", description: self.copy_stats.description },
            HelpBinding { keys: "T", description: self.cycle_theme.description },
            HelpBinding { keys: "L", description: self.cycle_layout.description },
            HelpBinding { keys: "a", description: self.sound_picker.description },
            HelpBinding { keys: "S", description: self.toggle_zen.description },
            HelpBinding { keys: "e", description: self.cycle_effects.description },
            HelpBinding { keys: "v", description: self.cycle_verb.description },
            HelpBinding { keys: "r", description: self.reset.description },
            HelpBinding { keys: "Esc", description: self.back.description },
            HelpBinding { keys: "q", description: self.quit.description },
            HelpBinding { keys: "?", description: self.help.description },
        ]
    }
}

