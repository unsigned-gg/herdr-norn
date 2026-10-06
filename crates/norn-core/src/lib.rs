//! norn-core — the capture engine of the norn project.
//!
//! Polls a herdr session's agent roster, reads each pane's rendered text, and
//! appends genuinely-new lines (norn_domain::new_lines) to the per-project
//! merged feed and the saga narrative. Shells out to the herdr CLI; no
//! sockets, no credentials, no writes outside the state dirs.

use norn_domain::{new_lines, slug, slug_from_remote, PaneEvent, RosterEntry};
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub struct Config {
    pub herdr: String,
    pub session: Option<String>,
    pub poll: Duration,
    pub read_lines: usize,
    pub feeds_dir: PathBuf,
    pub saga_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let state = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/state")
            });
        let data = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        Self {
            herdr: std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into()),
            session: std::env::var("NORN_SESSION").ok(),
            poll: Duration::from_secs_f64(
                std::env::var("NORN_POLL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2.0),
            ),
            read_lines: std::env::var("NORN_READ_LINES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            feeds_dir: state.join("norn/feeds"),
            saga_dir: data.join("saga"),
        }
    }
}

fn run(argv: &[String]) -> Option<String> {
    let out = Command::new(&argv[0]).args(&argv[1..]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

/// The session's live agents. A failed roster returns empty, never a guess:
/// a caller treating an error as empty would write nothing, which is safe.
pub fn roster(cfg: &Config) -> Vec<RosterEntry> {
    let mut argv = vec![cfg.herdr.clone()];
    if let Some(s) = &cfg.session {
        argv.extend(["--session".into(), s.clone()]);
    }
    argv.extend(["agent".into(), "list".into()]);
    let Some(text) = run(&argv) else {
        return vec![];
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return vec![];
    };
    v.pointer("/result/agents")
        .and_then(|a| a.as_array())
        .map(|agents| {
            agents
                .iter()
                .filter_map(|a| {
                    Some(RosterEntry {
                        pane_id: a.get("pane_id")?.as_str()?.to_string(),
                        agent: a.get("agent")?.as_str()?.to_string(),
                        cwd: a.get("cwd").and_then(|c| c.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// org/repo slug from the pane cwd's origin remote; the directory name when
/// there is no git remote.
pub fn project_of(cwd: Option<&str>) -> String {
    let Some(cwd) = cwd else {
        return "unknown".into();
    };
    if let Some(url) = run(&[
        "git".into(),
        "-C".into(),
        cwd.into(),
        "remote".into(),
        "get-url".into(),
        "origin".into(),
    ]) {
        if let Some(slug) = slug_from_remote(&url) {
            return slug;
        }
    }
    PathBuf::from(cwd)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".into())
}

/// The pane's rendered text as lines. `--source visible` reads the screen even
/// while the agent works; an agent-less pane answers nothing.
pub fn read_pane(cfg: &Config, pane: &str) -> Vec<String> {
    let mut argv = vec![cfg.herdr.clone()];
    if let Some(s) = &cfg.session {
        argv.extend(["--session".into(), s.clone()]);
    }
    argv.extend([
        "agent".into(),
        "read".into(),
        pane.into(),
        "--source".into(),
        "visible".into(),
        "--lines".into(),
        cfg.read_lines.to_string(),
    ]);
    run(&argv)
        .map(|t| t.lines().map(|l| l.trim_end().to_string()).collect())
        .unwrap_or_default()
}

fn stamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

pub fn feed_path(cfg: &Config, project: &str) -> PathBuf {
    cfg.feeds_dir.join(format!("{}.log", slug(project)))
}

pub fn saga_path(cfg: &Config, project: &str) -> PathBuf {
    cfg.saga_dir
        .join(slug(project))
        .join(format!("{}.md", today()))
}

/// One captured pane's new lines into both stores: the live feed (one line per
/// rendered line, tagged) and the saga narrative (one entry per batch).
pub fn append_event(cfg: &Config, ev: &PaneEvent) -> std::io::Result<()> {
    create_dir_all(&cfg.feeds_dir)?;
    let mut feed = OpenOptions::new()
        .create(true)
        .append(true)
        .open(feed_path(cfg, &ev.project))?;
    for ln in &ev.lines {
        writeln!(feed, "{} {}:{} | {}", stamp(), ev.agent, ev.pane_id, ln)?;
    }
    let saga = saga_path(cfg, &ev.project);
    if let Some(parent) = saga.parent() {
        create_dir_all(parent)?;
    }
    let mut log = OpenOptions::new().create(true).append(true).open(&saga)?;
    writeln!(log, "\n## {} — {} ({})\n", stamp(), ev.agent, ev.pane_id)?;
    for ln in &ev.lines {
        writeln!(log, "> {ln}")?;
    }
    Ok(())
}

/// One pass over the roster. Returns the events appended.
pub fn poll_once(cfg: &Config, seen: &mut HashMap<String, Vec<String>>) -> Vec<PaneEvent> {
    let mut events = vec![];
    for entry in roster(cfg) {
        let curr = read_pane(cfg, &entry.pane_id);
        let prev = seen.entry(entry.pane_id.clone()).or_default();
        let fresh: Vec<String> = new_lines(prev, &curr).to_vec();
        *prev = curr;
        if fresh.is_empty() {
            continue;
        }
        let ev = PaneEvent {
            project: project_of(entry.cwd.as_deref()),
            agent: entry.agent,
            pane_id: entry.pane_id,
            lines: fresh,
        };
        if append_event(cfg, &ev).is_ok() {
            events.push(ev);
        }
    }
    events
}

/// The capture daemon. Runs forever; meant for a supervisor (systemd user
/// unit / LaunchAgent), not a pane — the estate's sidecar rule. A bad pass
/// never kills the recorder.
pub fn record(cfg: &Config) -> ! {
    let mut seen: HashMap<String, Vec<String>> = HashMap::new();
    println!(
        "norn record: session={} poll={:?} feeds={}",
        cfg.session.as_deref().unwrap_or("default"),
        cfg.poll,
        cfg.feeds_dir.display()
    );
    loop {
        let pass =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| poll_once(cfg, &mut seen)));
        if let Err(e) = pass {
            eprintln!("norn record: pass failed: {e:?}");
        }
        std::thread::sleep(cfg.poll);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_domain::{PaneEvent, RosterEntry};

    fn tmp_cfg() -> (std::path::PathBuf, Config) {
        let root = std::env::temp_dir().join(format!("norn-core-test-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let cfg = Config {
            herdr: "herdr".into(),
            session: None,
            poll: Duration::from_secs(2),
            read_lines: 60,
            feeds_dir: root.join("feeds"),
            saga_dir: root.join("saga"),
        };
        (root, cfg)
    }

    #[test]
    fn append_event_writes_feed_and_saga() {
        let (root, cfg) = tmp_cfg();
        let ev = PaneEvent {
            project: "cerebral-work/cortex".into(),
            agent: "opencode".into(),
            pane_id: "w7:p1".into(),
            lines: vec!["line one".into(), "line two".into()],
        };
        append_event(&cfg, &ev).unwrap();
        let feed = std::fs::read_to_string(feed_path(&cfg, "cerebral-work/cortex")).unwrap();
        assert!(feed.contains("opencode:w7:p1 | line one"));
        assert!(feed.contains("opencode:w7:p1 | line two"));
        let saga = std::fs::read_to_string(saga_path(&cfg, "cerebral-work/cortex")).unwrap();
        assert!(saga.contains("opencode (w7:p1)"));
        assert!(saga.contains("> line two"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn append_event_keeps_projects_apart() {
        let (root, cfg) = tmp_cfg();
        let mk = |project: &str, line: &str| PaneEvent {
            project: project.into(),
            agent: "omp".into(),
            pane_id: "w7:p7".into(),
            lines: vec![line.into()],
        };
        append_event(&cfg, &mk("a/one", "for-one")).unwrap();
        append_event(&cfg, &mk("b/two", "for-two")).unwrap();
        let one = std::fs::read_to_string(feed_path(&cfg, "a/one")).unwrap();
        let two = std::fs::read_to_string(feed_path(&cfg, "b/two")).unwrap();
        assert!(one.contains("for-one") && !one.contains("for-two"));
        assert!(two.contains("for-two") && !two.contains("for-one"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn roster_entry_carries_cwd_for_project_mapping() {
        let e = RosterEntry {
            pane_id: "w1:p1".into(),
            agent: "claude".into(),
            cwd: Some("/Users/x/projects/cerebral-work/cortex".into()),
        };
        assert_eq!(
            e.cwd.as_deref().unwrap().rsplit('/').next().unwrap(),
            "cortex"
        );
    }
}
