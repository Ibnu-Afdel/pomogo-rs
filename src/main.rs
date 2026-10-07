use std::io;
use std::process;

use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use clap_complete::{generate, Shell};

use pomogo_rust::config::{db_file_path, Config};
use pomogo_rust::integrations::{format_status, run_doctor};
use pomogo_rust::omarchy::{install_desktop_entry, install_quickshell_plugin, print_omarchy_status};
use pomogo_rust::render::ambient::render_ambient;
use pomogo_rust::render::bigclock::ansi_fg;
use pomogo_rust::render::{
    resolve_effects_name, resolve_layout, resolve_layout_name, DisplayState, Frame,
};
use pomogo_rust::statefile::StateManager;
use pomogo_rust::stats::calculate as calculate_stats;
use pomogo_rust::store::models::Project;
use pomogo_rust::store::Store;
use pomogo_rust::theme::{get as get_theme, list as list_themes, resolve_theme_name};
use pomogo_rust::timer::SessionPhase;
use pomogo_rust::ui::App;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "pomogo", version = VERSION, about = "Sleek, distraction-free Pomodoro timer TUI for Linux, Omarchy, and developers")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version information
    Version,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Show focus statistics
    Stats {
        /// Show weekly activity
        #[arg(long)]
        week: bool,
        /// Show monthly summary
        #[arg(long)]
        month: bool,
    },
    /// Show detailed session history
    History,
    /// List all available color themes with swatches
    Themes,
    /// Render a non-interactive terminal preview
    ScreenshotPreview {
        #[arg(long, default_value = "monolith")]
        layout: String,
        #[arg(long, default_value = "daily")]
        theme: String,
        #[arg(long, default_value = "none")]
        effects: String,
        #[arg(long, default_value_t = 80)]
        width: usize,
        #[arg(long, default_value_t = 24)]
        height: usize,
        #[arg(long)]
        zen: bool,
    },
    /// Show summary of the last completed block
    Recap,
    /// Show current session status for Waybar, Tmux, or scripts
    Status {
        /// Output format (default, waybar, tmux, json)
        #[arg(long, default_value = "default")]
        format: String,
    },
    /// Generate shell completion scripts
    Completion {
        /// Shell (bash, zsh, fish)
        shell: String,
    },
    /// Manage focus projects
    Projects {
        #[command(subcommand)]
        action: Option<ProjectAction>,
    },
    /// Start timer with a specific profile or project
    Start {
        /// Target profile or project name
        target: String,
    },
    /// Check system dependencies and configuration health
    Doctor,
    /// Export focus session history to JSON or CSV format
    Export {
        /// Output format (json or csv)
        #[arg(long, default_value = "json")]
        format: String,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
    /// Generate a weekly focus report in Markdown
    Report {
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
    /// First-party Omarchy Linux integration commands
    Omarchy {
        #[command(subcommand)]
        action: Option<OmarchyAction>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Create a default config file
    Init,
}

#[derive(Subcommand)]
enum ProjectAction {
    /// List all projects
    List,
    /// Add a new project
    Add {
        name: String,
        color: Option<String>,
        #[arg(long)]
        icon: Option<String>,
    },
    /// Archive an existing project
    Archive {
        name: String,
    },
}

#[derive(Subcommand)]
enum OmarchyAction {
    /// Display Omarchy integration health and palette information
    Status,
    /// Install Quickshell top-bar widget into Omarchy plugins
    InstallQuickshell,
    /// Install Pomogo desktop launcher entry
    InstallDesktop,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => handle_default(),
        Some(Commands::Version) => handle_version(),
        Some(Commands::Config { action }) => handle_config(action),
        Some(Commands::Stats { week, month }) => handle_stats(week, month),
        Some(Commands::History) => handle_history(),
        Some(Commands::Themes) => handle_themes(),
        Some(Commands::ScreenshotPreview { layout, theme, effects, width, height, zen }) => {
            handle_screenshot_preview(&layout, &theme, &effects, width, height, zen);
        }
        Some(Commands::Recap) => handle_recap(),
        Some(Commands::Status { format }) => handle_status(&format),
        Some(Commands::Completion { shell }) => handle_completion(&shell),
        Some(Commands::Projects { action }) => handle_projects(action),
        Some(Commands::Start { target }) => handle_start(&target),
        Some(Commands::Doctor) => handle_doctor(),
        Some(Commands::Export { format, start, end }) => handle_export(&format, start, end),
        Some(Commands::Report { start, end }) => handle_report(start, end),
        Some(Commands::Omarchy { action }) => handle_omarchy(action),
    }
}

fn handle_default() {
    let cfg = Config::load().unwrap_or_default();
    let mut app = App::new(cfg);
    if let Err(e) = app.run() {
        eprintln!("Error running PomoGo: {}", e);
        process::exit(1);
    }
}

fn handle_version() {
    println!("pomogo-rust {} (Linux & Omarchy Edition)", VERSION);
}

fn handle_config(action: ConfigAction) {
    match action {
        ConfigAction::Init => match Config::write_default() {
            Ok(path) => println!("Config file created at: {}", path.display()),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
    }
}

fn handle_stats(week: bool, month: bool) {
    let store = match Store::new(&db_file_path()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    let now = Utc::now();
    let start = now - Duration::days(365);
    let end = now + Duration::days(1);
    let sessions = store.get_sessions(start, end).unwrap_or_default();

    let s = calculate_stats(&sessions, Local::now(), None);

    if week {
        println!("Weekly Focus Activity (mins):");
        println!("-----------------------------");
        for wd in &s.week_days {
            let mut bar_len = wd.minutes / 10;
            if bar_len > 15 {
                bar_len = 15;
            }
            let bar = if bar_len > 0 { "█".repeat(bar_len) } else { "░".to_string() };
            println!("{}  {:<15} {}m", wd.date.format("%a"), bar, wd.minutes);
        }
        println!();
        println!("Total Week Completed: {} sessions ({} mins)", s.week_count, s.week_minutes);
        return;
    }

    if month {
        println!("Monthly Focus Summary:");
        println!("----------------------");
        println!("This Month Completed: {} sessions", s.month_count);
        println!("Focus Completion Rate: {:.0}%", s.completion_rate * 100.0);
        return;
    }

    println!("PomoGo Focus Statistics:");
    println!("------------------------");
    println!("Today Completed:    {} sessions ({} mins focused)", s.today_count, s.today_minutes);
    println!("Current Streak:     {} days", s.current_streak);
    println!("Best Streak:        {} days", s.best_streak);
    println!("Monthly Completed:  {} sessions (Rate: {:.0}%)", s.month_count, s.completion_rate * 100.0);
    println!(
        "Lifetime Completed: {} sessions ({}h {}m focused)",
        s.lifetime_sessions,
        s.lifetime_minutes / 60,
        s.lifetime_minutes % 60
    );
}

fn handle_history() {
    let store = match Store::new(&db_file_path()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    let now = Utc::now();
    let start = now - Duration::days(365);
    let end = now + Duration::days(1);
    let sessions = store.get_sessions(start, end).unwrap_or_default();

    println!("Recent Work Sessions:");
    println!("---------------------");

    let mut count = 0;
    for s in sessions.iter().rev() {
        if s.session_type != "work" {
            continue;
        }
        let status = if s.completed { "completed" } else { "skipped" };
        let task = s.task.as_deref().unwrap_or("[no task]");
        let time_str = s.started_at.with_timezone(&Local).format("%Y-%m-%d %H:%M");

        print!("{}  {:<15} ({})", time_str, task, status);
        if let Some(note) = &s.note {
            if !note.is_empty() {
                print!(" - {}", note);
            }
        }
        println!();

        count += 1;
        if count >= 20 {
            break;
        }
    }

    if count == 0 {
        println!("No work sessions recorded yet.");
    }
}

fn handle_themes() {
    let themes = list_themes();
    println!("Available Color Themes:");
    println!("-----------------------");

    for name in themes {
        let t = get_theme(&name);
        let swatch = ansi_fg(t.work(), "■ Work");
        let brk = ansi_fg(t.brk(), "■ Break");
        let acc = ansi_fg(t.accent(), "■ Accent");
        println!("  {:<16} {}  {}  {}  - {}", t.name, swatch, brk, acc, t.description);
    }
}

fn handle_screenshot_preview(
    layout_flag: &str,
    theme_flag: &str,
    effects_flag: &str,
    width: usize,
    height: usize,
    zen: bool,
) {
    if width < 40 || height < 10 {
        eprintln!("Error: preview size must be at least 40x10");
        process::exit(1);
    }

    let resolved_th = resolve_theme_name(theme_flag);
    let resolved_ly = resolve_layout_name(layout_flag);
    let resolved_eff = resolve_effects_name(effects_flag);

    let th = get_theme(&resolved_th);
    let frame = Frame { width, height };
    let (l_name, layout_fn) = resolve_layout(&resolved_ly, frame.width, frame.height);

    let ds = DisplayState {
        mode_label: "Building".to_string(),
        project: "PomoGo".to_string(),
        task: "refine terminal focus".to_string(),
        phase_kind: SessionPhase::Work,
        segment_remaining: Duration::minutes(18) + Duration::seconds(32),
        block_remaining: Duration::hours(2) + Duration::minutes(12) + Duration::seconds(9),
        progress: 0.42,
        segment_index: 2,
        segment_count: 4,
        running: true,
        paused: false,
        idle: false,
        status_message: "focus · break in 19m".to_string(),
        hints_visibility: true,
        theme_name: resolved_th,
        layout_name: l_name.to_string(),
        zen,
        git_branch: "feature/focus-polish".to_string(),
        tmux_session: "work".to_string(),
    };

    let rendered = layout_fn(&ds, &th, &frame);
    print!("{}", render_ambient(&resolved_eff, 42, frame.width, frame.height, &th, &rendered));
}

fn handle_recap() {
    let store = match Store::new(&db_file_path()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    match store.get_last_block() {
        Ok(Some(b)) => {
            let mode_str = if b.mode == "deep" { "Deep Focus" } else { "Quick Focus" };
            let hrs = b.planned_secs / 3600;
            let mins = (b.planned_secs % 3600) / 60;
            let secs = b.planned_secs % 60;
            let time_str = if hrs > 0 {
                format!("{}h {}m", hrs, mins)
            } else if mins > 0 {
                format!("{}m {}s", mins, secs)
            } else {
                format!("{}s", secs)
            };
            let status_str = if b.completed { "Completed" } else { "Abandoned" };

            println!("Last Focus Cycle Recap:");
            println!("-----------------------");
            println!("  Mode:            {}", mode_str);
            println!("  Status:          {}", status_str);
            println!("  Started At:      {}", b.started_at.with_timezone(&Local).format("%Y-%m-%d %H:%M"));
            println!("  Planned Time:    {}", time_str);
            println!("  Pauses Taken:    {}", b.pauses);
        }
        _ => {
            println!("No completed blocks found.");
        }
    }
}

fn handle_status(format: &str) {
    let mgr = StateManager::new().ok();
    let state = mgr.and_then(|m| m.read().ok().flatten());
    match format_status(state.as_ref(), format) {
        Ok(out) => println!("{}", out),
        Err(_) => println!("Idle"),
    }
}

fn handle_completion(shell_name: &str) {
    let shell = match shell_name.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        other => {
            eprintln!("Unsupported shell: {}. Supported: bash, zsh, fish", other);
            process::exit(1);
        }
    };

    let mut cmd = <Cli as clap::CommandFactory>::command();
    generate(shell, &mut cmd, "pomogo", &mut io::stdout());
}

fn handle_projects(action: Option<ProjectAction>) {
    let mut store = match Store::new(&db_file_path()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    match action.unwrap_or(ProjectAction::List) {
        ProjectAction::List => {
            let projects = store.get_projects().unwrap_or_default();
            if projects.is_empty() {
                println!("No projects found.");
                return;
            }

            let mut active = Vec::new();
            let mut archived = Vec::new();

            for p in projects {
                let color_suffix = if !p.color.is_empty() {
                    format!(" ({})", p.color)
                } else {
                    String::new()
                };
                let display = if !p.icon.is_empty() {
                    format!("{} {}", p.icon, p.name)
                } else {
                    p.name
                };

                if p.archived {
                    archived.push(format!("{}{}", display, color_suffix));
                } else {
                    active.push(format!("{}{}", display, color_suffix));
                }
            }

            println!("Active Projects:");
            if active.is_empty() {
                println!("  (none)");
            } else {
                for a in active {
                    println!("  - {}", a);
                }
            }

            if !archived.is_empty() {
                println!("\nArchived Projects:");
                for a in archived {
                    println!("  - {} (archived)", a);
                }
            }
        }
        ProjectAction::Add { name, color, icon } => {
            let mut p = Project {
                id: 0,
                name: name.clone(),
                color: color.unwrap_or_default(),
                archived: false,
                icon: icon.unwrap_or_default(),
            };
            if let Err(e) = store.create_project(&mut p) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            println!("Project '{}' added successfully.", name);
        }
        ProjectAction::Archive { name } => {
            if let Err(e) = store.archive_project(&name) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            println!("Project '{}' archived successfully.", name);
        }
    }
}

fn handle_start(target: &str) {
    let mut cfg = Config::load().unwrap_or_default();
    let (resolved_cfg, mut project, sound_event) = cfg.resolve_profile(target);
    cfg = resolved_cfg;

    if project.is_empty() {
        if let Ok(store) = Store::new(&db_file_path()) {
            if let Ok(Some(p)) = store.get_project_by_name(target) {
                project = p.name;
            }
        }
    }
    if project.is_empty() {
        project = target.to_string();
    }

    let mut app = App::new(cfg);
    app.set_project_by_name(&project);
    if !sound_event.is_empty() {
        app.notifier.set_sound_events(sound_event.clone(), sound_event);
    }

    if let Err(e) = app.run() {
        eprintln!("Error running PomoGo: {}", e);
        process::exit(1);
    }
}

fn handle_doctor() {
    println!("Checking PomoGo system health...");
    println!();

    let diags = run_doctor();
    let mut all_passed = true;

    for d in diags {
        let status = if d.passed { "✔" } else { "✘" };
        if !d.passed {
            all_passed = false;
        }
        println!("[{}] {:<40} : {}", status, d.name, d.message);
    }
    println!();

    if all_passed {
        println!("All systems normal! PomoGo is fully operational.");
    } else {
        println!("Some warnings/errors were detected. Please check the reports above.");
    }
}

fn handle_export(format: &str, start_str: Option<String>, end_str: Option<String>) {
    let store = match Store::new(&db_file_path()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: failed to open store: {}", e);
            process::exit(1);
        }
    };

    let start = parse_date_or(start_str.as_deref(), Utc::now() - Duration::days(365));
    let end = parse_date_or(end_str.as_deref(), Utc::now() + Duration::days(1));

    match store.export_sessions(format, start, end) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: export failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_report(start_str: Option<String>, end_str: Option<String>) {
    let store = match Store::new(&db_file_path()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: failed to open store: {}", e);
            process::exit(1);
        }
    };

    let start = parse_date_or(start_str.as_deref(), Utc::now() - Duration::days(7));
    let end = parse_date_or(end_str.as_deref(), Utc::now());

    match store.generate_markdown_report(start, end) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: report generation failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_omarchy(action: Option<OmarchyAction>) {
    match action.unwrap_or(OmarchyAction::Status) {
        OmarchyAction::Status => {
            print_omarchy_status();
        }
        OmarchyAction::InstallQuickshell => match install_quickshell_plugin() {
            Ok(p) => println!("✔ Quickshell plugin installed successfully at: {}", p.display()),
            Err(e) => {
                eprintln!("Error installing Quickshell plugin: {}", e);
                process::exit(1);
            }
        },
        OmarchyAction::InstallDesktop => match install_desktop_entry() {
            Ok(p) => println!("✔ Desktop launcher installed at: {}", p.display()),
            Err(e) => {
                eprintln!("Error installing desktop file: {}", e);
                process::exit(1);
            }
        },
    }
}

fn parse_date_or(date_str: Option<&str>, default: DateTime<Utc>) -> DateTime<Utc> {
    if let Some(s) = date_str {
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return DateTime::from_naive_utc_and_offset(dt, Utc);
            }
        }
        eprintln!("Invalid date format {:?}. Use YYYY-MM-DD.", s);
        process::exit(1);
    }
    default
}
