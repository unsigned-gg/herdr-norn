//! saga — the durable narrative: what actually happened across a project's
//! agents, kept as dated markdown you can read, diff, and commit.
//!
//!   saga read <project> [--all]    today's (or every) narrative for a project
//!   saga watch <project>           follow today's narrative live (the pane entrypoint)
//!   saga projects                  list projects with narratives

use norn_core::{saga_path, Config};
use norn_domain::slug;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::process::exit;

fn read(project: &str, all_days: bool) {
    let cfg = Config::default();
    let root = cfg.saga_dir.join(slug(project));
    if !root.is_dir() {
        eprintln!("saga: no narrative for {project} yet — start `norn record` first");
        exit(2);
    }
    let mut days: Vec<_> = if all_days {
        std::fs::read_dir(&root)
            .map(|d| d.flatten().map(|e| e.path()).collect())
            .unwrap_or_default()
    } else {
        vec![saga_path(&cfg, project)]
    };
    days.sort();
    let mut found = false;
    for day in days {
        if day.is_file() {
            found = true;
            let stem = day.file_stem().unwrap_or_default().to_string_lossy();
            println!("\x1b[1m# {project} — {stem}\x1b[0m");
            print!("{}", std::fs::read_to_string(&day).unwrap_or_default());
        }
    }
    if !found {
        eprintln!("saga: nothing written for {project} today (try --all)");
        exit(1);
    }
}

fn watch(project: &str) -> ! {
    let cfg = Config::default();
    let path = saga_path(&cfg, project);
    println!(
        "\x1b[1msaga · {project}\x1b[0m  ({})  — ctrl+c to detach",
        path.display()
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    let mut f = BufReader::new(std::fs::File::open(&path).expect("saga file just created"));
    f.seek(SeekFrom::End(0)).ok();
    let mut line = String::new();
    loop {
        line.clear();
        match f.read_line(&mut line) {
            Ok(0) => std::thread::sleep(std::time::Duration::from_millis(500)),
            Ok(_) => {
                print!("{line}");
                std::io::stdout().flush().ok();
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(500)),
        }
    }
}

fn projects(cfg: &Config) {
    if let Ok(dir) = std::fs::read_dir(&cfg.saga_dir) {
        let mut rows: Vec<_> = dir.flatten().collect();
        rows.sort_by_key(|e| e.file_name());
        for p in rows {
            println!("{}", p.file_name().to_string_lossy().replace("__", "/"));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = Config::default();
    match args.first().map(String::as_str) {
        Some("read") => {
            let Some(project) = args.get(1).filter(|a| !a.starts_with('-')) else {
                eprintln!("saga read needs a project (saga projects lists them)");
                exit(2);
            };
            read(project, args.iter().any(|a| a == "--all"));
        }
        Some("watch") => {
            let Some(project) = args.get(1) else {
                eprintln!("saga watch needs a project");
                exit(2);
            };
            watch(project)
        }
        Some("projects") => projects(&cfg),
        _ => {
            eprintln!("usage: saga read <project> [--all] | saga watch <project> | saga projects");
            exit(2);
        }
    }
}
