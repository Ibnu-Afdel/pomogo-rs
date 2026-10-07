pub mod keymap;
pub mod screens;

use std::io::{stdout, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use chrono::{DateTime, Duration, Local, Utc};
use crossterm::{
    cursor,
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, Clear, ClearType,
        EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
use signal_hook::flag as signal_flag;

use crate::config::{db_file_path, Config};
use crate::devinfo::{find_git_branch, get_tmux_session};
use crate::integrations::is_session_locked;
use crate::notify::{sound_profiles, Notifier};
use crate::render::ambient::render_ambient;
use crate::render::text::clip_visible;
use crate::render::{
    resolve_effects_name, resolve_layout, resolve_layout_name, DisplayState, Frame,
};
use crate::restore::{can_restore, restore_runner, RestoreDurations};
use crate::session::{Block, Mode, Runner, RunnerEventType};
use crate::statefile::StateManager;
use crate::stats::calculate as calculate_stats;
use crate::store::models::{BlockStore, DbSession};
use crate::store::Store;
use crate::theme::omarchy::omarchy_colors_mtime;
use crate::theme::{self, Theme};
use crate::timer::{RealClock, SessionPhase, SessionState};
use crate::ui::keymap::KeyMap;
use crate::ui::screens::{
    preset_duration, render_duration_picker, render_help, render_input, render_recap,
    render_restore_prompt, render_sound_picker, render_stats, RecapInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    TaskInput,
    NoteInput,
    ProjectInput,
    DurationPicker,
    CustomDurationInput,
    SoundPicker,
    RecapScreen,
}

pub struct App {
    pub runner: Runner,
    pub cfg: Config,
    pub theme: Theme,
    pub width: usize,
    pub height: usize,
    pub keymap: KeyMap,

    pub notifier: Notifier,
    pub state_manager: Option<StateManager>,
    pub db_store: Option<Store>,
    pub restore_pending: bool,
    pub show_help: bool,
    pub show_stats: bool,
    pub status_message: String,
    pub status_clear_at: Option<Instant>,

    pub input_mode: InputMode,
    pub input_text: String,
    pub current_task: String,
    pub pending_session: Option<DbSession>,
    pub current_project_id: Option<i64>,
    pub current_project_name: String,
    pub suggestions: Vec<String>,
    pub filtered_suggestions: Vec<String>,
    pub suggestion_index: i32,

    pub selected_mode: Mode,
    pub selected_duration_idx: usize,
    pub selected_sound_idx: usize,
    pub deep_duration: Duration,
    pub current_block_id: Option<i64>,

    pub current_theme_name: String,
    pub current_layout_name: String,
    pub current_effects_name: String,
    pub zen_mode: bool,
    pub tick_count: usize,
    pub current_verb_label: String,
    pub git_branch: String,
    pub tmux_session: String,

    pub paused_by_lock: bool,
    pub last_lock_check: Instant,
    pub recap_info: Option<RecapInfo>,
    /// Last seen mtime of Omarchy's colors.toml, for following `omarchy theme set`.
    pub omarchy_palette_mtime: Option<SystemTime>,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let current_theme_name = theme::resolve_theme_name(&cfg.theme);
        let current_layout_name = resolve_layout_name(&cfg.layout);
        let current_effects_name = resolve_effects_name(&cfg.effects);
        let th = theme::get(&current_theme_name);

        let state_manager = StateManager::new().ok();
        let db_store = Store::new(&db_file_path()).ok();

        let restore_pending = can_restore();

        let mut current_task = String::new();
        let mut current_project_id = None;
        let mut current_project_name = String::new();

        if let Some(mgr) = &state_manager {
            if let Ok(Some(st)) = mgr.read() {
                if let Some(t) = st.task {
                    current_task = t;
                }
                current_project_id = st.project_id;
                if let Some(p) = st.project_name {
                    current_project_name = p;
                }
            }
        }

        let git_branch = if cfg.show_git {
            find_git_branch(&std::env::current_dir().unwrap_or_default()).unwrap_or_default()
        } else {
            String::new()
        };

        let tmux_session = if cfg.show_tmux {
            get_tmux_session().unwrap_or_default()
        } else {
            String::new()
        };

        let notifier = Notifier::new(
            cfg.notifications_enabled,
            cfg.sound_enabled,
            Some(cfg.sound_start_event.clone()),
            Some(cfg.sound_end_event.clone()),
        );

        let block = Block::new_quick(
            cfg.quick_focus_work_duration_as_duration(),
            cfg.quick_focus_short_break_duration_as_duration(),
            cfg.quick_focus_long_break_duration_as_duration(),
            cfg.quick_focus_sessions_before_long_break(),
            cfg.quick_focus_auto_advance(),
        );

        let verb = get_verb_for_task(&current_task);

        Self {
            runner: Runner::new(block),
            cfg,
            theme: th,
            width: 80,
            height: 24,
            keymap: KeyMap::default(),
            notifier,
            state_manager,
            db_store,
            restore_pending,
            show_help: false,
            show_stats: false,
            status_message: String::new(),
            status_clear_at: None,
            input_mode: InputMode::None,
            input_text: String::new(),
            current_task,
            pending_session: None,
            current_project_id,
            current_project_name,
            suggestions: Vec::new(),
            filtered_suggestions: Vec::new(),
            suggestion_index: -1,
            selected_mode: Mode::Quick,
            selected_duration_idx: 1, // 120m default
            selected_sound_idx: 0,
            deep_duration: Duration::hours(2),
            current_block_id: None,
            current_theme_name,
            current_layout_name,
            current_effects_name,
            zen_mode: false,
            tick_count: 0,
            current_verb_label: verb,
            git_branch,
            tmux_session,
            paused_by_lock: false,
            last_lock_check: Instant::now(),
            recap_info: None,
            omarchy_palette_mtime: omarchy_colors_mtime(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Leave the terminal usable if anything below panics.
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), cursor::Show, LeaveAlternateScreen);
            default_hook(info);
        }));

        // Closing the terminal window (SIGHUP) or a plain `kill` should save
        // the session like `q` does; SIGUSR1/SIGUSR2 are `pomogo toggle`/`skip`.
        let quit_flag = Arc::new(AtomicBool::new(false));
        for sig in [SIGTERM, SIGHUP, SIGINT] {
            signal_flag::register(sig, Arc::clone(&quit_flag))?;
        }
        let toggle_flag = Arc::new(AtomicBool::new(false));
        signal_flag::register(SIGUSR1, Arc::clone(&toggle_flag))?;
        let skip_flag = Arc::new(AtomicBool::new(false));
        signal_flag::register(SIGUSR2, Arc::clone(&skip_flag))?;

        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        if let Ok((cols, rows)) = crossterm::terminal::size() {
            self.width = cols as usize;
            self.height = rows as usize;
        }

        self.write_state();

        let mut last_tick = Instant::now();
        let tick_interval = StdDuration::from_millis(250);

        let clock = RealClock;

        loop {
            if quit_flag.load(Ordering::Relaxed) {
                break;
            }
            if toggle_flag.swap(false, Ordering::Relaxed) {
                let key = if self.runner.timer.is_running { ' ' } else { 's' };
                self.handle_main_key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE))?;
            }
            if skip_flag.swap(false, Ordering::Relaxed) {
                self.handle_main_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE))?;
            }

            // Check status message timeout
            if let Some(t) = self.status_clear_at {
                if Instant::now() >= t {
                    self.status_message.clear();
                    self.status_clear_at = None;
                }
            }

            // Check system lock every 3 seconds if enabled
            if self.cfg.pause_on_lock && self.last_lock_check.elapsed() >= StdDuration::from_secs(3) {
                self.last_lock_check = Instant::now();
                let locked = is_session_locked();
                if locked && self.runner.timer.is_running && !self.runner.timer.is_paused {
                    let _ = self.runner.timer.pause(&clock);
                    self.paused_by_lock = true;
                    self.write_state();
                } else if !locked && self.paused_by_lock && self.runner.timer.is_paused {
                    let _ = self.runner.timer.resume(&clock);
                    self.paused_by_lock = false;
                    self.write_state();
                }
            }

            // Render UI
            self.render_frame(&mut stdout)?;

            // Event poll
            let timeout = tick_interval
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| StdDuration::from_millis(10));

            if event::poll(timeout)? {
                match event::read()? {
                    CEvent::Key(key) => {
                        if self.handle_key(key)? {
                            break; // Quit
                        }
                    }
                    CEvent::Resize(cols, rows) => {
                        self.width = cols as usize;
                        self.height = rows as usize;
                    }
                    _ => {}
                }
            }

            // Tick handling every ~1s (4 * 250ms)
            if last_tick.elapsed() >= StdDuration::from_secs(1) {
                last_tick = Instant::now();
                self.tick_count += 1;
                self.follow_omarchy_theme();

                if self.runner.timer.is_running && !self.runner.timer.is_paused {
                    let prev_phase = self.runner.timer.phase;
                    let started_at = self.runner.timer.started_at.unwrap_or_else(Utc::now);

                    if let Some(evt) = self.runner.tick(&clock) {
                        let dur = self.runner.block.current_segment.duration;
                        self.record_session(prev_phase, started_at, Utc::now(), true, dur);

                        // Trigger notifications & sounds
                        self.notifier.notify_transition(evt.state, evt.phase);

                        // Run user hook
                        self.run_hook_for_transition(evt.phase);

                        if evt.event_type == RunnerEventType::BlockEnded {
                            self.finish_block(true);
                            if self.selected_mode == Mode::Deep && self.cfg.prompt_for_notes {
                                self.pending_session = Some(DbSession {
                                    id: 0,
                                    session_type: "work".to_string(),
                                    task: Some(self.current_task.clone()),
                                    note: None,
                                    started_at,
                                    ended_at: Some(Utc::now()),
                                    completed: true,
                                    duration_secs: dur.num_seconds(),
                                    project_id: self.current_project_id,
                                    project_name: Some(self.current_project_name.clone()),
                                    mode: Some("deep".to_string()),
                                    block_id: self.current_block_id,
                                });
                                self.input_mode = InputMode::NoteInput;
                                self.input_text.clear();
                            } else {
                                self.show_recap_screen();
                            }
                        } else if self.selected_mode == Mode::Quick && prev_phase == SessionPhase::LongBreak {
                            self.show_recap_screen();
                        }
                    }

                    self.update_terminal_title();
                }

                // Heartbeat: bar widgets treat a state file that stops
                // updating as "PomoGo is no longer running".
                self.write_state();
            }
        }

        // Cleanup
        self.persist_on_quit();
        disable_raw_mode()?;
        execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        if self.zen_mode {
            if key.code == KeyCode::Char('q') {
                return Ok(true);
            }
            self.zen_mode = false;
            return Ok(false);
        }

        if self.restore_pending {
            return self.handle_restore_key(key);
        }

        if self.show_help {
            match key.code {
                KeyCode::Char('?') | KeyCode::Esc => self.show_help = false,
                KeyCode::Char('q') => return Ok(true),
                _ => {}
            }
            return Ok(false);
        }

        if self.show_stats {
            match key.code {
                KeyCode::Tab | KeyCode::Esc => self.show_stats = false,
                KeyCode::Char('y') => self.copy_stats_to_clipboard(),
                KeyCode::Char('q') => return Ok(true),
                _ => {}
            }
            return Ok(false);
        }

        match self.input_mode {
            InputMode::None => self.handle_main_key(key),
            InputMode::DurationPicker => self.handle_duration_picker_key(key),
            InputMode::CustomDurationInput => self.handle_custom_duration_key(key),
            InputMode::SoundPicker => self.handle_sound_picker_key(key),
            InputMode::TaskInput | InputMode::ProjectInput | InputMode::NoteInput => {
                self.handle_text_input_key(key)
            }
            InputMode::RecapScreen => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                    self.input_mode = InputMode::None;
                }
                Ok(false)
            }
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        let clock = RealClock;
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab => self.show_stats = !self.show_stats,
            KeyCode::Char('y') => self.copy_stats_to_clipboard(),
            KeyCode::Char('s') => {
                if !self.runner.timer.is_running {
                    if self.selected_mode == Mode::Deep && self.db_store.is_some() {
                        let mut b_store = BlockStore {
                            id: 0,
                            mode: "deep".to_string(),
                            planned_secs: self.deep_duration.num_seconds(),
                            started_at: Utc::now(),
                            ended_at: None,
                            completed: false,
                            pauses: 0,
                        };
                        if let Some(store) = &mut self.db_store {
                            if store.create_block(&mut b_store).is_ok() {
                                self.current_block_id = Some(b_store.id);
                            }
                        }
                    }

                    if let Err(e) = self.runner.start(&clock) {
                        self.set_status(&e.to_string());
                    } else {
                        self.run_hook_for_transition(self.runner.timer.phase);
                        self.write_state();
                    }
                }
            }
            KeyCode::Char(' ') => {
                if self.runner.timer.is_running {
                    if self.runner.timer.is_paused {
                        let _ = self.runner.timer.resume(&clock);
                    } else {
                        let _ = self.runner.timer.pause(&clock);
                        if self.selected_mode == Mode::Deep {
                            if let (Some(id), Some(store)) = (self.current_block_id, &self.db_store) {
                                let _ = store.increment_block_pauses(id);
                            }
                        }
                    }
                    self.write_state();
                }
            }
            KeyCode::Char('n') => {
                if self.runner.timer.is_running || self.runner.timer.is_paused {
                    let prev_phase = self.runner.timer.phase;
                    let started_at = self.runner.timer.started_at.unwrap_or_else(Utc::now);
                    let dur = self.runner.block.current_segment.duration;

                    let (evt, _ok) = self.runner.skip(&clock);
                    self.record_session(prev_phase, started_at, Utc::now(), false, dur);

                    if evt.event_type == RunnerEventType::BlockEnded {
                        self.finish_block(false);
                        if self.selected_mode == Mode::Deep && self.cfg.prompt_for_notes {
                            self.input_mode = InputMode::NoteInput;
                            self.input_text.clear();
                        } else {
                            self.show_recap_screen();
                        }
                    } else if self.selected_mode == Mode::Quick && prev_phase == SessionPhase::LongBreak {
                        self.show_recap_screen();
                    }

                    self.write_state();
                }
            }
            KeyCode::Char('r') => {
                self.runner.timer.reset();
                self.finish_block(false);
                if let Some(mgr) = &self.state_manager {
                    let _ = mgr.remove();
                }
            }
            KeyCode::Char('t') => {
                self.input_mode = InputMode::TaskInput;
                self.input_text = self.current_task.clone();
                self.load_task_suggestions();
            }
            KeyCode::Char('p') => {
                self.input_mode = InputMode::ProjectInput;
                self.input_text = self.current_project_name.clone();
                self.load_project_suggestions();
            }
            KeyCode::Char('d') => {
                if !self.runner.timer.is_running {
                    self.input_mode = InputMode::DurationPicker;
                }
            }
            KeyCode::Char('T') => {
                self.cycle_theme();
            }
            KeyCode::Char('L') => {
                self.cycle_layout();
            }
            KeyCode::Char('a') => {
                self.input_mode = InputMode::SoundPicker;
            }
            KeyCode::Char('S') => {
                self.zen_mode = !self.zen_mode;
            }
            KeyCode::Char('e') => {
                self.cycle_effects();
            }
            KeyCode::Char('v') => {
                self.cycle_verb();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_duration_picker_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Esc => self.input_mode = InputMode::None,
            KeyCode::Char('1') => self.selected_duration_idx = 0,
            KeyCode::Char('2') => self.selected_duration_idx = 1,
            KeyCode::Char('3') => self.selected_duration_idx = 2,
            KeyCode::Char('4') => self.selected_duration_idx = 3,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_duration_idx == 0 {
                    self.selected_duration_idx = 4;
                } else {
                    self.selected_duration_idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.selected_duration_idx = (self.selected_duration_idx + 1) % 5;
            }
            KeyCode::Enter => {
                if self.selected_duration_idx <= 3 {
                    self.deep_duration = preset_duration(self.selected_duration_idx);
                    self.configure_deep_focus(self.deep_duration);
                    self.input_mode = InputMode::None;
                } else {
                    self.input_mode = InputMode::CustomDurationInput;
                    self.input_text.clear();
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_custom_duration_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Esc => self.input_mode = InputMode::DurationPicker,
            KeyCode::Enter => {
                if let Ok(d) = parse_custom_duration(&self.input_text) {
                    self.deep_duration = d;
                    self.configure_deep_focus(self.deep_duration);
                    self.input_mode = InputMode::None;
                } else {
                    self.set_status("Invalid duration format (e.g., 90m, 1h30m)");
                    self.input_mode = InputMode::DurationPicker;
                }
            }
            KeyCode::Backspace => {
                self.input_text.pop();
            }
            KeyCode::Char(c) => {
                self.input_text.push(c);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_sound_picker_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        let profiles = sound_profiles();
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Esc => self.input_mode = InputMode::None,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_sound_idx == 0 {
                    self.selected_sound_idx = profiles.len() - 1;
                } else {
                    self.selected_sound_idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.selected_sound_idx = (self.selected_sound_idx + 1) % profiles.len();
            }
            KeyCode::Char(' ') => {
                if self.selected_sound_idx < profiles.len() {
                    self.notifier.preview_sound_event(profiles[self.selected_sound_idx].start_event);
                }
            }
            KeyCode::Enter => {
                if self.selected_sound_idx < profiles.len() {
                    let p = &profiles[self.selected_sound_idx];
                    self.cfg.sound_start_event = p.start_event.to_string();
                    self.cfg.sound_end_event = p.end_event.to_string();
                    self.notifier.set_sound_events(p.start_event.to_string(), p.end_event.to_string());
                    self.set_status(&format!("Sound: {}", p.name));
                    self.input_mode = InputMode::None;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_text_input_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
            }
            KeyCode::Enter => {
                let text = self.input_text.trim().to_string();
                match self.input_mode {
                    InputMode::TaskInput => {
                        self.current_task = text.clone();
                        self.current_verb_label = get_verb_for_task(&text);
                        self.write_state();
                    }
                    InputMode::ProjectInput => {
                        self.current_project_name = text.clone();
                        if let Some(store) = &self.db_store {
                            if let Ok(Some(p)) = store.get_project_by_name(&text) {
                                self.current_project_id = Some(p.id);
                            } else {
                                self.current_project_id = None;
                            }
                        }
                        self.write_state();
                    }
                    InputMode::NoteInput => {
                        if let (Some(mut sess), Some(store)) = (self.pending_session.take(), &mut self.db_store) {
                            sess.note = if text.is_empty() { None } else { Some(text) };
                            let _ = store.save_session(&mut sess);
                        }
                        self.show_recap_screen();
                        return Ok(false);
                    }
                    _ => {}
                }
                self.input_mode = InputMode::None;
            }
            KeyCode::Backspace => {
                self.input_text.pop();
                self.filter_suggestions();
            }
            KeyCode::Down => {
                if !self.filtered_suggestions.is_empty() {
                    self.suggestion_index = (self.suggestion_index + 1).min(self.filtered_suggestions.len() as i32 - 1);
                }
            }
            KeyCode::Up => {
                if self.suggestion_index > -1 {
                    self.suggestion_index -= 1;
                }
            }
            KeyCode::Tab => {
                if self.suggestion_index >= 0 && (self.suggestion_index as usize) < self.filtered_suggestions.len() {
                    self.input_text = self.filtered_suggestions[self.suggestion_index as usize].clone();
                }
            }
            KeyCode::Char(c) => {
                self.input_text.push(c);
                self.filter_suggestions();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_restore_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let durations = RestoreDurations {
                    quick_work: self.cfg.quick_focus_work_duration_as_duration(),
                    quick_short_break: self.cfg.quick_focus_short_break_duration_as_duration(),
                    quick_long_break: self.cfg.quick_focus_long_break_duration_as_duration(),
                    quick_sessions_before_long_break: self.cfg.quick_focus_sessions_before_long_break(),
                    quick_auto_advance: self.cfg.quick_focus_auto_advance(),
                    deep_work: self.cfg.deep_focus_work_duration_as_duration(),
                    deep_short_break: self.cfg.deep_focus_short_break_duration_as_duration(),
                    deep_long_break: self.cfg.deep_focus_long_break_duration_as_duration(),
                    deep_sessions_before_long_break: self.cfg.deep_focus_sessions_before_long_break(),
                };

                match restore_runner(&durations) {
                    Ok(r) => {
                        self.runner = r;
                        self.restore_pending = false;
                        self.write_state();
                    }
                    Err(e) => {
                        self.restore_pending = false;
                        self.set_status(&format!("Restore failed: {}", e));
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.restore_pending = false;
                self.runner.timer.reset();
                self.write_state();
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    fn configure_deep_focus(&mut self, total: Duration) {
        let block = Block::new_deep(
            total,
            self.cfg.deep_focus_work_duration_as_duration(),
            self.cfg.deep_focus_short_break_duration_as_duration(),
            self.cfg.deep_focus_long_break_duration_as_duration(),
            self.cfg.deep_focus_sessions_before_long_break(),
            true,
        );
        self.runner = Runner::new(block);
        self.selected_mode = Mode::Deep;
        self.deep_duration = total;
        self.set_status(&format!("Deep Focus: {} min block", total.num_minutes()));
    }

    /// Reloads the palette when the Omarchy theme changes underneath us.
    fn follow_omarchy_theme(&mut self) {
        if self.current_theme_name != "omarchy" {
            return;
        }
        let mtime = omarchy_colors_mtime();
        if mtime != self.omarchy_palette_mtime {
            self.omarchy_palette_mtime = mtime;
            self.theme = theme::get("omarchy");
        }
    }

    fn cycle_theme(&mut self) {
        let all = theme::list();
        if let Some(pos) = all.iter().position(|t| t == &self.current_theme_name) {
            let next_pos = (pos + 1) % all.len();
            self.current_theme_name = all[next_pos].clone();
        } else {
            self.current_theme_name = all.first().cloned().unwrap_or_else(|| "tokyo-night".to_string());
        }
        self.theme = theme::get(&self.current_theme_name);
        self.set_status(&format!("Theme: {}", self.current_theme_name));
    }

    fn cycle_layout(&mut self) {
        let layouts = [
            "classic", "minimal", "centered", "compact", "retro", "dashboard",
            "monolith", "tinybar", "terminal-rice", "focus-stack", "command-center",
        ];
        if let Some(pos) = layouts.iter().position(|l| *l == self.current_layout_name.as_str()) {
            let next_pos = (pos + 1) % layouts.len();
            self.current_layout_name = layouts[next_pos].to_string();
        } else {
            self.current_layout_name = "classic".to_string();
        }
        self.set_status(&format!("Layout: {}", self.current_layout_name));
    }

    fn cycle_effects(&mut self) {
        let effects = ["none", "stars", "snow", "rain", "embers", "scanline"];
        if let Some(pos) = effects.iter().position(|e| *e == self.current_effects_name.as_str()) {
            let next_pos = (pos + 1) % effects.len();
            self.current_effects_name = effects[next_pos].to_string();
        } else {
            self.current_effects_name = "none".to_string();
        }
        self.set_status(&format!("Effects: {}", self.current_effects_name));
    }

    fn cycle_verb(&mut self) {
        let verbs = [
            "Building", "Focusing", "Writing", "Testing", "Reviewing",
            "Refactoring", "Designing", "Reading",
        ];
        if let Some(pos) = verbs.iter().position(|v| *v == self.current_verb_label.as_str()) {
            let next_pos = (pos + 1) % verbs.len();
            self.current_verb_label = verbs[next_pos].to_string();
        } else {
            self.current_verb_label = "Building".to_string();
        }
        self.set_status(&format!("Activity: {}", self.current_verb_label));
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_clear_at = Some(Instant::now() + StdDuration::from_secs(2));
    }

    fn copy_stats_to_clipboard(&mut self) {
        let sessions = self.db_store.as_ref().and_then(|s| {
            s.get_sessions(Utc::now() - Duration::days(365), Utc::now() + Duration::days(1)).ok()
        }).unwrap_or_default();

        let s = calculate_stats(&sessions, Local::now(), None);
        let summary = format!(
            "PomoGo Stats - Today: {} sessions ({} mins) | Streak: {} days | Month: {} sessions",
            s.today_count, s.today_minutes, s.current_streak, s.month_count
        );

        // Copy via arboard
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(&summary);
        }

        // OSC 52 fallback
        let b64 = base64_encode(&summary);
        print!("\x1b]52;c;{}\x07", b64);
        let _ = stdout().flush();

        self.set_status("Copied stats to clipboard!");
    }

    fn load_task_suggestions(&mut self) {
        self.suggestions.clear();
        if let Some(store) = &self.db_store {
            if let Ok(tasks) = store.get_unique_tasks(self.current_project_id) {
                self.suggestions = tasks;
            }
        }
        self.filtered_suggestions = self.suggestions.clone();
        self.suggestion_index = -1;
    }

    fn load_project_suggestions(&mut self) {
        self.suggestions.clear();
        if let Some(store) = &self.db_store {
            if let Ok(projects) = store.get_projects() {
                self.suggestions = projects.into_iter().filter(|p| !p.archived).map(|p| p.name).collect();
            }
        }
        self.filtered_suggestions = self.suggestions.clone();
        self.suggestion_index = -1;
    }

    fn filter_suggestions(&mut self) {
        let q = self.input_text.trim().to_lowercase();
        if q.is_empty() {
            self.filtered_suggestions = self.suggestions.clone();
        } else {
            self.filtered_suggestions = self
                .suggestions
                .iter()
                .filter(|s| s.to_lowercase().contains(&q))
                .cloned()
                .collect();
        }
        if self.suggestion_index >= self.filtered_suggestions.len() as i32 {
            self.suggestion_index = self.filtered_suggestions.len() as i32 - 1;
        }
    }

    fn record_session(
        &mut self,
        phase: SessionPhase,
        started_at: DateTime<Utc>,
        ended_at: DateTime<Utc>,
        completed: bool,
        duration: Duration,
    ) {
        if let Some(store) = &mut self.db_store {
            let type_str = match phase {
                SessionPhase::Work => "work",
                SessionPhase::ShortBreak => "short_break",
                SessionPhase::LongBreak => "long_break",
            };
            let mut sess = DbSession {
                id: 0,
                session_type: type_str.to_string(),
                task: Some(self.current_task.clone()),
                note: None,
                started_at,
                ended_at: Some(ended_at),
                completed,
                duration_secs: duration.num_seconds(),
                project_id: self.current_project_id,
                project_name: Some(self.current_project_name.clone()),
                mode: Some(self.runner.block.mode.as_str().to_string()),
                block_id: self.current_block_id,
            };
            let _ = store.save_session(&mut sess);
        }
    }

    fn finish_block(&mut self, completed: bool) {
        if self.runner.block.mode == Mode::Deep {
            if let (Some(id), Some(store)) = (self.current_block_id, &self.db_store) {
                let _ = store.finish_block(id, completed, Utc::now());
            }
        }
    }

    fn show_recap_screen(&mut self) {
        let sessions = self.db_store.as_ref().and_then(|s| {
            s.get_sessions(Utc::now() - Duration::days(365), Utc::now() + Duration::days(1)).ok()
        }).unwrap_or_default();
        let s = calculate_stats(&sessions, Local::now(), None);

        let info = RecapInfo {
            total_focused: Duration::minutes(s.today_minutes as i64),
            segments: self.runner.block.session_count,
            breaks: (self.runner.block.session_count).saturating_sub(1),
            pauses: 0,
            streak: s.current_streak,
            is_deep: self.selected_mode == Mode::Deep,
            focus_score: 9,
        };
        self.recap_info = Some(info);
        self.input_mode = InputMode::RecapScreen;
    }

    fn write_state(&self) {
        if let Some(mgr) = &self.state_manager {
            let _ = mgr.write(
                &self.runner,
                Some(&self.current_task),
                self.current_project_id,
                Some(&self.current_project_name),
                self.current_block_id,
            );
        }
    }

    fn persist_on_quit(&self) {
        self.write_state();
    }

    fn update_terminal_title(&self) {
        if !self.cfg.terminal_title_enabled {
            return;
        }
        let mins = self.runner.timer.remaining_time.num_minutes();
        let secs = self.runner.timer.remaining_time.num_seconds() % 60;
        let icon = match self.runner.timer.phase {
            SessionPhase::Work => "🍅",
            _ => "☕",
        };
        let title = format!("{} {:02}:{:02} - PomoGo", icon, mins, secs);
        print!("\x1b]0;{}\x07", title);
        let _ = stdout().flush();
    }

    fn run_hook_for_transition(&self, phase: SessionPhase) {
        let hook_cmd = match phase {
            SessionPhase::Work => &self.cfg.on_work_start,
            SessionPhase::ShortBreak | SessionPhase::LongBreak => &self.cfg.on_break_start,
        };

        if let Some(cmd_str) = hook_cmd {
            if !cmd_str.trim().is_empty() {
                let cmd_owned = cmd_str.clone();
                std::thread::spawn(move || {
                    let _ = Command::new("sh").args(["-c", &cmd_owned]).spawn();
                });
            }
        }
    }

    fn render_frame(&mut self, out: &mut std::io::Stdout) -> Result<(), std::io::Error> {
        let frame = Frame {
            width: self.width,
            height: self.height,
        };

        let rendered = if self.restore_pending {
            render_restore_prompt(self.width, self.height, &self.theme)
        } else if self.show_help {
            render_help(self.width, self.height, &self.theme, &self.keymap.help_bindings())
        } else if self.show_stats {
            let sessions = self.db_store.as_ref().and_then(|s| {
                s.get_sessions(Utc::now() - Duration::days(365), Utc::now() + Duration::days(1)).ok()
            }).unwrap_or_default();
            let s = calculate_stats(&sessions, Local::now(), None);
            render_stats(self.width, self.height, &self.theme, &s, &self.status_message, &sessions)
        } else {
            match self.input_mode {
                InputMode::DurationPicker => render_duration_picker(
                    self.width,
                    self.height,
                    &self.theme,
                    self.selected_duration_idx,
                    self.cfg.deep_focus_default_duration_as_duration(),
                    self.cfg.deep_focus_work_duration_as_duration(),
                    self.cfg.deep_focus_short_break_duration_as_duration(),
                    self.cfg.deep_focus_long_break_duration_as_duration(),
                    self.cfg.deep_focus_sessions_before_long_break(),
                ),
                InputMode::SoundPicker => {
                    render_sound_picker(self.width, self.height, &self.theme, self.selected_sound_idx, &sound_profiles())
                }
                InputMode::TaskInput => render_input(
                    self.width,
                    self.height,
                    &self.theme,
                    "task",
                    &self.input_text,
                    &self.filtered_suggestions,
                    self.suggestion_index,
                ),
                InputMode::ProjectInput => render_input(
                    self.width,
                    self.height,
                    &self.theme,
                    "project",
                    &self.input_text,
                    &self.filtered_suggestions,
                    self.suggestion_index,
                ),
                InputMode::NoteInput => render_input(
                    self.width,
                    self.height,
                    &self.theme,
                    "note",
                    &self.input_text,
                    &[],
                    -1,
                ),
                InputMode::CustomDurationInput => render_input(
                    self.width,
                    self.height,
                    &self.theme,
                    "custom_duration",
                    &self.input_text,
                    &[],
                    -1,
                ),
                InputMode::RecapScreen => {
                    if let Some(info) = &self.recap_info {
                        render_recap(self.width, self.height, &self.theme, info)
                    } else {
                        self.render_timer_layout(&frame)
                    }
                }
                InputMode::None => self.render_timer_layout(&frame),
            }
        };

        // Raw mode does not translate "\n" into "\r\n", so position every row
        // explicitly and clear whatever the previous frame left behind.
        queue!(out, BeginSynchronizedUpdate)?;
        let mut row = 0u16;
        for line in rendered.split('\n').take(self.height.max(1)) {
            queue!(out, cursor::MoveTo(0, row))?;
            write!(out, "{}\x1b[0m", clip_visible(line.trim_end_matches('\r'), self.width))?;
            queue!(out, Clear(ClearType::UntilNewLine))?;
            row += 1;
        }
        if (row as usize) < self.height {
            queue!(out, cursor::MoveTo(0, row), Clear(ClearType::FromCursorDown))?;
        }
        queue!(out, EndSynchronizedUpdate)?;
        out.flush()?;

        Ok(())
    }

    fn render_timer_layout(&self, frame: &Frame) -> String {
        let (resolved_name, layout_fn) = resolve_layout(&self.current_layout_name, frame.width, frame.height);

        let total_block = if self.runner.block.mode == Mode::Deep {
            self.runner.block.planned_total
        } else {
            self.runner.block.current_segment.duration
        };

        let progress = if total_block > Duration::zero() {
            let elapsed = total_block - self.runner.block.remaining(self.runner.timer.remaining_time);
            (elapsed.num_seconds() as f64 / total_block.num_seconds() as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let ds = DisplayState {
            mode_label: self.current_verb_label.clone(),
            project: self.current_project_name.clone(),
            task: self.current_task.clone(),
            phase_kind: self.runner.timer.phase,
            segment_remaining: self.runner.timer.remaining_time,
            block_remaining: if self.runner.block.mode == Mode::Deep {
                self.runner.block.remaining(self.runner.timer.remaining_time)
            } else {
                Duration::zero()
            },
            progress,
            segment_index: self.runner.block.index,
            segment_count: if self.runner.block.mode == Mode::Deep {
                self.runner.block.segments.len()
            } else {
                self.runner.block.sessions_before_long_break
            },
            paused: self.runner.timer.is_paused,
            running: self.runner.timer.is_running,
            idle: self.runner.timer.state == SessionState::Idle,
            status_message: if !self.status_message.is_empty() {
                self.status_message.clone()
            } else if self.runner.timer.is_paused {
                "paused".to_string()
            } else {
                format!("focus · break in {}m", (self.runner.timer.remaining_time.num_seconds() + 59) / 60)
            },
            hints_visibility: true,
            theme_name: self.current_theme_name.clone(),
            layout_name: resolved_name.to_string(),
            zen: self.zen_mode,
            git_branch: self.git_branch.clone(),
            tmux_session: self.tmux_session.clone(),
        };

        let layout_output = layout_fn(&ds, &self.theme, frame);
        render_ambient(
            &self.current_effects_name,
            self.tick_count,
            frame.width,
            frame.height,
            &self.theme,
            &layout_output,
        )
    }

    pub fn set_project_by_name(&mut self, project: &str) {
        self.current_project_name = project.to_string();
        if let Some(store) = &self.db_store {
            if let Ok(Some(p)) = store.get_project_by_name(project) {
                self.current_project_id = Some(p.id);
            }
        }
    }
}

fn get_verb_for_task(task: &str) -> String {
    let lower = task.trim().to_lowercase();
    if lower.starts_with("fix") {
        "Fixing".to_string()
    } else if lower.starts_with("build") {
        "Building".to_string()
    } else if lower.starts_with("test") {
        "Testing".to_string()
    } else if lower.starts_with("refactor") {
        "Refactoring".to_string()
    } else if lower.starts_with("review") {
        "Reviewing".to_string()
    } else if lower.starts_with("write") {
        "Writing".to_string()
    } else if lower.starts_with("read") {
        "Reading".to_string()
    } else if lower.starts_with("design") {
        "Designing".to_string()
    } else {
        "Focusing".to_string()
    }
}

fn parse_custom_duration(val: &str) -> Result<Duration, String> {
    let v = val.trim().to_lowercase();
    if v.is_empty() {
        return Err("empty duration".to_string());
    }

    // Try parsing e.g. "90m", "1h30m", "2h"
    let mut total_mins = 0;
    let mut cur_num = String::new();

    for ch in v.chars() {
        if ch.is_ascii_digit() {
            cur_num.push(ch);
        } else if ch == 'h' {
            let n: i64 = cur_num.parse().map_err(|_| "invalid number")?;
            total_mins += n * 60;
            cur_num.clear();
        } else if ch == 'm' {
            let n: i64 = cur_num.parse().map_err(|_| "invalid number")?;
            total_mins += n;
            cur_num.clear();
        }
    }

    if !cur_num.is_empty() {
        let n: i64 = cur_num.parse().map_err(|_| "invalid number")?;
        total_mins += n;
    }

    if total_mins > 0 {
        Ok(Duration::minutes(total_mins))
    } else {
        Err("duration must be positive".to_string())
    }
}

fn base64_encode(input: &str) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        let i0 = (b0 >> 2) as usize;
        let i1 = (((b0 & 0x03) << 4) | (b1 >> 4)) as usize;
        let i2 = (((b1 & 0x0f) << 2) | (b2 >> 6)) as usize;
        let i3 = (b2 & 0x3f) as usize;

        out.push(CHARSET[i0] as char);
        out.push(CHARSET[i1] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[i2] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[i3] as char);
        } else {
            out.push('=');
        }
    }

    out
}

