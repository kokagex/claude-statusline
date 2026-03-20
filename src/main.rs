use serde::Deserialize;
use std::io::Read;
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
    used_percentage: Option<u8>,
    context_window_size: Option<u64>,
}

#[derive(Deserialize)]
struct RateLimits {
    five_hour: Option<RateLimit>,
    seven_day: Option<RateLimit>,
}

#[derive(Deserialize)]
struct RateLimit {
    used_percentage: Option<u8>,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: Option<String>,
    project_dir: Option<String>,
}

fn pct_color(v: Option<u8>) -> &'static str {
    match v {
        None => GRAY,
        Some(v) if v < 50 => GREEN,
        Some(v) if v < 80 => YELLOW,
        _ => RED,
    }
}

fn pct_str(v: Option<u8>) -> String {
    match v {
        Some(v) => format!("{v}%"),
        None => "?%".to_string(),
    }
}

fn format_ctx_size(size: Option<u64>) -> String {
    match size {
        Some(s) if s >= 1_000_000 => format!("({}M)", s / 1_000_000),
        Some(s) if s >= 1_000 => format!("({}K)", s / 1_000),
        _ => String::new(),
    }
}

fn git_branch(cwd: &str) -> String {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
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
        .unwrap_or_else(|| "-".to_string())
}

fn main() {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return;
    }

    let d: Input = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return,
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
    let s5h = d
        .rate_limits
        .as_ref()
        .and_then(|r| r.five_hour.as_ref())
        .and_then(|r| r.used_percentage);
    let s7d = d
        .rate_limits
        .as_ref()
        .and_then(|r| r.seven_day.as_ref())
        .and_then(|r| r.used_percentage);
    let cwd = d
        .workspace
        .as_ref()
        .and_then(|w| w.project_dir.as_deref().or(w.current_dir.as_deref()))
        .unwrap_or(".");
    let branch = git_branch(cwd);

    let sep = format!("{DIM} | {RESET}");
    print!(
        "{BOLD}{BLURPLE}{model}{RESET}{sep}\
         {GRAY}CTX:{RESET}{c1}{v1}{RESET}{sep}\
         {GRAY}5h:{RESET}{c2}{v2}{RESET} {GRAY}7d:{RESET}{c3}{v3}{RESET}{sep}\
         {BLURPLE}{branch}{RESET}",
        c1 = pct_color(ctx),
        v1 = pct_str(ctx),
        c2 = pct_color(s5h),
        v2 = pct_str(s5h),
        c3 = pct_color(s7d),
        v3 = pct_str(s7d),
    );
}
