// `pomogo setup`: a one-time set of questions that writes config.toml.
// It also runs on the very first launch so PomoGo can run on its own after.

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

use crate::config::{config_file_path, write_config, SetupChoices};
use crate::omarchy::{install_plugin, plugin_dir};
use crate::theme::omarchy::is_omarchy_environment;
use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::theme::{self, Theme};

struct Rhythm {
    name: &'static str,
    blurb: &'static str,
    work: usize,
    short_break: usize,
    long_break: usize,
    every: usize,
}

const RHYTHMS: [Rhythm; 3] = [
    Rhythm { name: "Classic", blurb: "25 min focus, 5 min breaks, 15 min every 4", work: 25, short_break: 5, long_break: 15, every: 4 },
    Rhythm { name: "Steady", blurb: "50 min focus, 10 min breaks, 20 min every 3", work: 50, short_break: 10, long_break: 20, every: 3 },
    Rhythm { name: "Deep", blurb: "90 min focus, 20 min breaks, 30 min every 2", work: 90, short_break: 20, long_break: 30, every: 2 },
];

/// Whether setup can ask questions here (both ends are a terminal).
pub fn interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// True on the very first launch: no config has been written yet.
pub fn first_run() -> bool {
    !config_file_path().exists()
}

struct Prompter<'a> {
    th: &'a Theme,
    input: io::StdinLock<'static>,
}

impl Prompter<'_> {
    fn ask(&mut self, question: &str, hint: &str) -> io::Result<String> {
        print!("{} {} ", ansi_bold_fg(self.th.text(), question), ansi_fg(self.th.muted(), hint));
        io::stdout().flush()?;
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "setup cancelled"));
        }
        Ok(line.trim().to_string())
    }

    fn yes_no(&mut self, question: &str, default: bool) -> io::Result<bool> {
        loop {
            let hint = if default { "[Y/n]" } else { "[y/N]" };
            match self.ask(question, hint)?.to_lowercase().as_str() {
                "" => return Ok(default),
                "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                _ => println!("{}", ansi_fg(self.th.muted(), "  Please answer y or n.")),
            }
        }
    }

    fn choice(&mut self, question: &str, count: usize, default: usize) -> io::Result<usize> {
        loop {
            let answer = self.ask(question, &format!("[1-{}, enter for {}]", count, default))?;
            if answer.is_empty() {
                return Ok(default);
            }
            match answer.parse::<usize>() {
                Ok(n) if (1..=count).contains(&n) => return Ok(n),
                _ => println!("{}", ansi_fg(self.th.muted(), &format!("  Pick a number from 1 to {}.", count))),
            }
        }
    }

    fn hours(&mut self, question: &str, default: f64) -> io::Result<f64> {
        loop {
            let answer = self.ask(question, &format!("[hours, enter for {}, 0 for none]", default))?;
            if answer.is_empty() {
                return Ok(default);
            }
            match answer.replace(',', ".").parse::<f64>() {
                Ok(h) if (0.0..=16.0).contains(&h) => return Ok(h),
                _ => println!("{}", ansi_fg(self.th.muted(), "  Enter a number of hours, like 4 or 2.5.")),
            }
        }
    }
}

fn section(th: &Theme, step: usize, total: usize, title: &str) {
    println!();
    println!("{} {}", ansi_fg(th.muted(), &format!("{}/{}", step, total)), ansi_bold_fg(th.accent(), title));
}

/// Offer the bar widget on Omarchy when it isn't installed yet.
fn offer_widget() -> bool {
    is_omarchy_environment() && !plugin_dir().join("manifest.json").exists()
}

/// Asks the setup questions and writes config.toml. Returns the path written.
pub fn run(force: bool) -> Result<PathBuf, String> {
    let th = theme::get(&theme::resolve_theme_name("auto"));
    let mut p = Prompter { th: &th, input: io::stdin().lock() };
    let io_err = |e: io::Error| e.to_string();
    let widget = offer_widget();
    let total = if widget { 6 } else { 5 };

    println!();
    println!("{}", ansi_bold_fg(th.work(), "PomoGo setup"));
    println!(
        "{}",
        ansi_fg(
            th.muted(),
            "Answer once and PomoGo runs your focus day on its own. Enter keeps the default."
        )
    );

    section(&th, 1, total, "Focus rhythm");
    for (i, r) in RHYTHMS.iter().enumerate() {
        println!(
            "  {} {:<8}{}",
            ansi_bold_fg(th.accent(), &format!("{}", i + 1)),
            r.name,
            ansi_fg(th.muted(), r.blurb)
        );
    }
    let rhythm = &RHYTHMS[p.choice("Which rhythm suits your work?", RHYTHMS.len(), 1).map_err(io_err)? - 1];

    section(&th, 2, total, "Daily goal");
    let goal_hours = p.hours("How much focused time do you aim for each day?", 4.0).map_err(io_err)?;

    section(&th, 3, total, "Autopilot");
    println!("{}", ansi_fg(th.muted(), "  Breaks start on their own, and so does the next focus after a break."));
    let autopilot = p.yes_no("Run focus and breaks automatically?", true).map_err(io_err)?;

    section(&th, 4, total, "Body reminders");
    println!("{}", ansi_fg(th.muted(), "  While you focus: rest your eyes every 20 min, water every 45, stretch every 60."));
    let reminders = p.yes_no("Remind you to look after yourself?", true).map_err(io_err)?;

    section(&th, 5, total, "Notifications");
    let notifications = p.yes_no("Show desktop notifications and play sounds?", true).map_err(io_err)?;

    let add_widget = if widget {
        section(&th, 6, total, "Omarchy bar");
        println!("{}", ansi_fg(th.muted(), "  A small countdown in your bar: click to open, right click to pause."));
        p.yes_no("Add the PomoGo widget to your bar?", true).map_err(io_err)?
    } else {
        false
    };

    let choices = SetupChoices {
        work: rhythm.work,
        short_break: rhythm.short_break,
        long_break: rhythm.long_break,
        long_break_every: rhythm.every,
        daily_goal_minutes: (goal_hours * 60.0).round() as u32,
        autopilot,
        reminders,
        notifications,
    };
    let path = write_config(&choices, force)?;

    println!();
    println!("{} {}", ansi_bold_fg(th.long_break(), "✔ Saved"), ansi_fg(th.muted(), &path.display().to_string()));
    if add_widget {
        if let Err(e) = install_plugin(true) {
            println!("{}", ansi_fg(th.muted(), &format!("  Could not add the widget: {}", e)));
        }
    }
    Ok(path)
}
