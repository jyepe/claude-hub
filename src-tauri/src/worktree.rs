use std::path::{Path, PathBuf};

pub fn parent_repo(cwd: &Path) -> Option<PathBuf> {
    let dot_git = cwd.join(".git");
    let meta = std::fs::metadata(&dot_git).ok()?;
    if meta.is_dir() {
        return None;
    }
    let contents = std::fs::read_to_string(&dot_git).ok()?;
    let line = contents.lines().next()?;
    let gitdir_value = line.strip_prefix("gitdir:")?.trim();
    let gitdir = if Path::new(gitdir_value).is_absolute() {
        PathBuf::from(gitdir_value)
    } else {
        cwd.join(gitdir_value)
    };
    let canonical = gitdir.canonicalize().ok()?;
    let mut iter = canonical.iter().collect::<Vec<_>>();
    while let Some(seg) = iter.last() {
        if seg.to_string_lossy() == ".git" {
            iter.pop();
            return Some(iter.iter().collect());
        }
        iter.pop();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fresh_temp() -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "claude-hub-wt-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn returns_none_for_normal_repo() {
        let tmp = fresh_temp();
        std::fs::create_dir_all(tmp.join(".git")).unwrap();
        assert!(parent_repo(&tmp).is_none());
    }

    #[test]
    fn returns_none_for_non_git_dir() {
        let tmp = fresh_temp();
        assert!(parent_repo(&tmp).is_none());
    }

    #[test]
    fn resolves_worktree_to_parent() {
        let parent = fresh_temp();
        std::fs::create_dir_all(parent.join(".git/worktrees/feat")).unwrap();
        let wt = fresh_temp();
        let mut f = std::fs::File::create(wt.join(".git")).unwrap();
        let gitdir = parent.join(".git/worktrees/feat");
        write!(f, "gitdir: {}\n", gitdir.to_string_lossy()).unwrap();
        let resolved = parent_repo(&wt).unwrap();
        assert_eq!(resolved, parent.canonicalize().unwrap());
    }
}
