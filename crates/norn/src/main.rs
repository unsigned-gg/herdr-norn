//! norn — the capture daemon and parent tool. It watches a herdr session's
//! agent panes and feeds both consumers: skald (the live merged feed) and
//! saga (the durable narrative).
//!
//!   norn record [--session S]   capture daemon (run supervised, never a pane)
//!   norn projects               projects with captured feeds

use norn_core::Config;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = Config::default();
    match args.first().map(String::as_str) {
        Some("record") => {
            let mut cfg = cfg;
            if let Some(i) = args.iter().position(|a| a == "--session") {
                cfg.session = args.get(i + 1).cloned();
            }
            norn_core::record(&cfg)
        }
        Some("projects") => {
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
            eprintln!("usage: norn record [--session S] | norn projects");
            std::process::exit(2);
        }
    }
}
