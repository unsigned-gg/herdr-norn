//! skald — the live merged feed: every agent pane of a project, one stream.
//! The Norse reciter: it tells the work as it happens.
//!
//!   skald view [--project P]   watch the merged feed (the herdr pane entrypoint)
//!   skald projects             list projects with live feeds

use norn_core::{feed_path, project_of, Config};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::process::exit;

/// A small ANSI palette; the agent tag gets the color, the text stays plain.
const PALETTE: &[&str] = &[
    "31", "32", "33", "34", "35", "36", "91", "92", "93", "94", "95", "96",
];

fn color(agent: &str) -> &'static str {
    let h = agent
        .bytes()
        .fold(0usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
    PALETTE[h % PALETTE.len()]
}

/// The project of the pane the user was in when the action fired, from the
/// context herdr injects.
fn context_project() -> Option<String> {
    let raw = std::env::var("HERDR_PLUGIN_CONTEXT_JSON").ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let cwd = ["focused_pane_cwd", "workspace_cwd", "cwd"]
        .iter()
        .find_map(|k| v.get(k).and_then(|c| c.as_str()))?;
    Some(project_of(Some(cwd)))
}

fn latest_project(cfg: &Config) -> Option<String> {
    let mut newest: Option<(std::time::SystemTime, String)> = None;
    for entry in std::fs::read_dir(&cfg.feeds_dir).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".log") {
            continue;
        }
        let mtime = entry.metadata().ok()?.modified().ok()?;
        if newest.as_ref().is_none_or(|(t, _)| mtime > *t) {
            newest = Some((mtime, name.trim_end_matches(".log").replace("__", "/")));
        }
    }
    newest.map(|(_, n)| n)
}

fn view(project: Option<String>) -> ! {
    let cfg = Config::default();
    let project = project
        .or_else(context_project)
        .or_else(|| latest_project(&cfg));
    let Some(project) = project else {
        eprintln!("skald: no project named, none in context, and no feeds yet — start `norn record` first");
        exit(2);
    };
    let path = feed_path(&cfg, &project);
    println!(
        "\x1b[1mskald · {project}\x1b[0m  ({})  — ctrl+c to detach",
        path.display()
    );
    // Create on first view so tailing never races record.
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    let mut f = BufReader::new(std::fs::File::open(&path).expect("feed just created"));
    f.seek(SeekFrom::End(0)).ok();
    let mut line = String::new();
    loop {
        line.clear();
        match f.read_line(&mut line) {
            Ok(0) => std::thread::sleep(std::time::Duration::from_millis(500)),
            Ok(_) => {
                // "HH:MM:SS agent:pane | text" — color the agent tag only.
                let (tag, text) = line.split_once(" | ").unwrap_or((line.trim_end(), ""));
                let agent = tag
                    .split(' ')
                    .nth(1)
                    .and_then(|t| t.split(':').next())
                    .unwrap_or("");
                print!("\x1b[{c}m{tag}\x1b[0m | {text}", c = color(agent));
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(500)),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("view") | None => {
            let project = args
                .iter()
                .position(|a| a == "--project")
                .and_then(|i| args.get(i + 1).cloned());
            view(project)
        }
        Some("projects") => {
            let cfg = Config::default();
            let mut rows: Vec<_> = std::fs::read_dir(&cfg.feeds_dir)
                .map(|d| d.flatten().collect())
                .unwrap_or_default();
            rows.sort_by_key(|e| e.file_name());
            for p in rows {
                let name = p.file_name().to_string_lossy().into_owned();
                println!("{}", name.trim_end_matches(".log").replace("__", "/"));
            }
        }
        _ => {
            eprintln!("usage: skald view [--project P] | skald projects");
            exit(2);
        }
    }
}
