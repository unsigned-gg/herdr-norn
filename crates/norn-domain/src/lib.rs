//! norn-domain — the domain types and pure logic of the norn project.
//!
//! No I/O here: roster entries arrive parsed, lines leave as events. The
//! overlap diff is the load-bearing piece — agent panes repaint wholesale,
//! and a wrong diff either double-writes (garbage) or drops work (lies).

/// One pane in a herdr session's roster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterEntry {
    pub pane_id: String,
    pub agent: String,
    pub cwd: Option<String>,
}

/// One captured event: new rendered lines from one pane of one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneEvent {
    pub project: String,
    pub agent: String,
    pub pane_id: String,
    pub lines: Vec<String>,
}

/// The feed path's filesystem-safe project key.
pub fn slug(project: &str) -> String {
    project.replace('/', "__")
}

/// org/repo slug from an origin remote URL; two clones of one repo are ONE
/// project. Handles git@host:org/repo(.git) and https://host/org/repo forms.
pub fn slug_from_remote(url: &str) -> Option<String> {
    let url = url.trim();
    let bytes = url.as_bytes();
    for i in (0..bytes.len()).rev() {
        let c = bytes[i] as char;
        if c == ':' || c == '/' {
            let tail = &url[i + 1..];
            if let Some((org, repo)) = tail.rsplit_once('/') {
                if !org.is_empty() && !repo.is_empty() {
                    let repo = repo.strip_suffix(".git").unwrap_or(repo);
                    return Some(format!("{org}/{repo}"));
                }
            }
        }
    }
    None
}

/// Lines in `curr` that are genuinely new, by longest prev-suffix /
/// curr-prefix overlap.
///
/// Identical screens append nothing. A wholesale repaint (no overlap at all)
/// appends nothing — interleaved garbage is worse than a missed frame. This
/// is the contract the record daemon and both consumers rely on.
pub fn new_lines<'a>(prev: &[String], curr: &'a [String]) -> &'a [String] {
    if curr.is_empty() || prev == curr || prev.is_empty() {
        return &[];
    }
    let top = prev.len().min(curr.len());
    for k in (1..=top).rev() {
        if prev[prev.len() - k..] == curr[..k] {
            return &curr[k..];
        }
    }
    &[]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn appends_only_the_new_tail() {
        let prev = v(&["a", "b", "c"]);
        let curr = v(&["b", "c", "d", "e"]);
        assert_eq!(new_lines(&prev, &curr), &v(&["d", "e"]));
    }

    #[test]
    fn identical_screens_append_nothing() {
        let s = v(&["a", "b"]);
        assert!(new_lines(&s, &s).is_empty());
    }

    #[test]
    fn a_repaint_appends_nothing() {
        let prev = v(&["a", "b", "c"]);
        let curr = v(&["x", "y", "z"]);
        assert!(
            new_lines(&prev, &curr).is_empty(),
            "a full repaint must not dump"
        );
    }

    #[test]
    fn an_empty_prev_appends_nothing() {
        // First contact: the whole screen is not "new work".
        let curr = v(&["a"]);
        assert!(new_lines(&[], &curr).is_empty());
    }

    #[test]
    fn scroll_off_keeps_the_overlap() {
        let prev = v(&["a", "b", "c", "d"]);
        let curr = v(&["c", "d", "e"]);
        assert_eq!(new_lines(&prev, &curr), &v(&["e"]));
    }

    #[test]
    fn slug_maps_org_repo() {
        assert_eq!(
            slug_from_remote("git@github.com:cerebral-work/cortex.git"),
            Some("cerebral-work/cortex".into())
        );
        assert_eq!(
            slug_from_remote("https://github.com/unsigned-gg/herdr-norn"),
            Some("unsigned-gg/herdr-norn".into())
        );
        assert_eq!(slug_from_remote(""), None);
    }

    #[test]
    fn feed_slug_is_filesystem_safe() {
        assert_eq!(slug("cerebral-work/cortex"), "cerebral-work__cortex");
    }
}
