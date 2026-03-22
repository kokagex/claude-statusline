use chrono::{Local, TimeZone};
use serde::Deserialize;
use std::io::Read as IoRead;
use std::process::Command;

// Discord-inspired ANSI colors
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BLURPLE: &str = "\x1b[38;2;88;101;242m";
const GREEN: &str = "\x1b[38;2;87;242;135m";
const YELLOW: &str = "\x1b[38;2;254;231;92m";
const RED: &str = "\x1b[38;2;237;66;69m";
const GRAY: &str = "\x1b[38;2;148;155;164m";

#[derive(Deserialize)]
struct Input {
    model: Option<Model>,
    context_window: Option<ContextWindow>,
    rate_limits: Option<RateLimits>,
    workspace: Option<Workspace>,
}

#[derive(Deserialize)]
struct Model {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct ContextWindow {
    used_percentage: Option<f64>,
    context_window_size: Option<u64>,
}

#[derive(Deserialize)]
struct RateLimits {
    five_hour: Option<RateLimit>,
    seven_day: Option<RateLimit>,
}

#[derive(Deserialize)]
struct RateLimit {
    used_percentage: Option<f64>,
    resets_at: Option<i64>,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: Option<String>,
    project_dir: Option<String>,
}

fn pct_color(v: Option<f64>) -> &'static str {
    match v {
        None => GRAY,
        Some(v) if v < 50.0 => GREEN,
        Some(v) if v < 80.0 => YELLOW,
        _ => RED,
    }
}

fn pct_str(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{:.0}%", v),
        None => "?%".to_string(),
    }
}

fn format_reset_time(ts: Option<i64>) -> String {
    match ts {
        Some(t) => Local
            .timestamp_opt(t, 0)
            .single()
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "?".to_string()),
        None => "?".to_string(),
    }
}

fn format_reset_date(ts: Option<i64>) -> String {
    match ts {
        Some(t) => Local
            .timestamp_opt(t, 0)
            .single()
            .map(|dt| dt.format("%m/%d").to_string())
            .unwrap_or_else(|| "?".to_string()),
        None => "?".to_string(),
    }
}

fn format_ctx_size(size: Option<u64>) -> String {
    match size {
        Some(s) if s >= 1_000_000 => format!("({}M)", s / 1_000_000),
        Some(s) if s >= 1_000 => format!("({}K)", s / 1_000),
        _ => String::new(),
    }
}

fn try_git(args: &[&str], cwd: &str) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
}

fn git_branch(cwd: &str) -> String {
    // --no-optional-locks prevents git from failing or blocking when
    // .git/HEAD.lock is held during a branch switch (especially on Windows).
    try_git(
        &["--no-optional-locks", "rev-parse", "--abbrev-ref", "HEAD"],
        cwd,
    )
    // Fallback: symbolic-ref works even when rev-parse is mid-transition.
    .or_else(|| {
        try_git(
            &["--no-optional-locks", "symbolic-ref", "--short", "HEAD"],
            cwd,
        )
    })
    // Last resort: detached HEAD state — show short commit hash.
    .or_else(|| {
        try_git(
            &["--no-optional-locks", "rev-parse", "--short", "HEAD"],
            cwd,
        )
        .map(|h| format!("({})", h))
    })
    .unwrap_or_else(|| "-".to_string())
}

fn main() {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        eprint!("[statusline] stdin read failed");
        return;
    }

    let d: Input = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => {
            print!("{BOLD}{BLURPLE}Opti{RESET}{DIM} | {RESET}{GRAY}starting...{RESET}");
            return;
        }
    };

    let raw_name = d
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("?");
    let short_name = raw_name.split(' ').next().unwrap_or(raw_name);
    let ctx_label = format_ctx_size(
        d.context_window
            .as_ref()
            .and_then(|c| c.context_window_size),
    );
    let model = if ctx_label.is_empty() {
        short_name.to_string()
    } else {
        format!("{short_name} {ctx_label}")
    };

    let ctx = d.context_window.as_ref().and_then(|c| c.used_percentage);
    let rl5h = d.rate_limits.as_ref().and_then(|r| r.five_hour.as_ref());
    let s5h = rl5h.and_then(|r| r.used_percentage);
    let r5h = format_reset_time(rl5h.and_then(|r| r.resets_at));
    let rl7d = d.rate_limits.as_ref().and_then(|r| r.seven_day.as_ref());
    let s7d = rl7d.and_then(|r| r.used_percentage);
    let r7d = format_reset_date(rl7d.and_then(|r| r.resets_at));
    let branch = d
        .workspace
        .as_ref()
        .and_then(|w| w.project_dir.as_deref().or(w.current_dir.as_deref()))
        .map(|cwd| git_branch(cwd))
        .unwrap_or_else(|| "-".to_string());

    let sep = format!("{DIM} | {RESET}");
    print!(
        "{BOLD}{BLURPLE}{model}{RESET}{sep}\
         {BLURPLE}{branch}{RESET}\n\
         {GRAY}CTX:{RESET}{c1}{v1}{RESET}{sep}\
         {GRAY}5h:{RESET}{c2}{v2}{RESET}{DIM} ({r5h}){RESET}{sep}\
         {GRAY}7d:{RESET}{c3}{v3}{RESET}{DIM} ({r7d}){RESET}",
        c1 = pct_color(ctx),
        v1 = pct_str(ctx),
        c2 = pct_color(s5h),
        v2 = pct_str(s5h),
        c3 = pct_color(s7d),
        v3 = pct_str(s7d),
    );
}
