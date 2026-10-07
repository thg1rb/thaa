//! Shared, read-only Git CLI adapter for repository context discovery.

use std::env;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::git_context::{GitBranch, GitContext, GitContextProvider};

/// Uses the installed Git executable with fixed commands and structured args.
#[derive(Debug, Clone)]
pub struct GitCliContextProvider {
    executable: std::ffi::OsString,
}

impl Default for GitCliContextProvider {
    fn default() -> Self {
        Self {
            executable: "git".into(),
        }
    }
}

impl GitContextProvider for GitCliContextProvider {
    fn context_for(&self, working_directory: &Path) -> Option<GitContext> {
        let root = self.run(working_directory, &["rev-parse", "--show-toplevel"])?;
        let branch = self.run(working_directory, &["branch", "--show-current"])?;
        let repository_root = std::path::PathBuf::from(root);
        if !repository_root.is_absolute() {
            return None;
        }

        let branch = if branch.is_empty() {
            GitBranch::DetachedHead
        } else {
            GitBranch::Named(branch)
        };

        Some(GitContext {
            repository_root,
            branch,
        })
    }
}

impl GitCliContextProvider {
    const COMMAND_TIMEOUT: Duration = Duration::from_millis(750);
    const MAX_OUTPUT_BYTES: u64 = 64 * 1024;

    fn run(&self, working_directory: &Path, arguments: &[&str]) -> Option<String> {
        let mut command = Command::new(&self.executable);
        command
            .arg("-C")
            .arg(working_directory)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0");

        // Git environment variables can redirect discovery to another worktree,
        // index, or repository. The process cwd is the sole discovery input.
        for (key, _) in env::vars_os() {
            if key
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("GIT_")
            {
                command.env_remove(key);
            }
        }
        command
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0");

        let mut child = command.spawn().ok()?;
        let stdout = child.stdout.take()?;
        let mut output_reader = Some(thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout
                .take(Self::MAX_OUTPUT_BYTES + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
        }));
        let mut collected_output = None;
        let deadline = Instant::now() + Self::COMMAND_TIMEOUT;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let bytes = match collected_output {
                        Some(bytes) => bytes,
                        None => output_reader.take()?.join().ok()?.ok()?,
                    };
                    if !status.success() || bytes.len() as u64 > Self::MAX_OUTPUT_BYTES {
                        return None;
                    }
                    return Self::decode_output(bytes);
                }
                Ok(None) if Instant::now() < deadline => {
                    if collected_output.is_none()
                        && output_reader
                            .as_ref()
                            .is_some_and(|reader| reader.is_finished())
                    {
                        let bytes = match output_reader.take()?.join().ok()?.ok() {
                            Some(bytes) => bytes,
                            None => {
                                Self::kill_and_reap(&mut child);
                                return None;
                            }
                        };
                        if bytes.len() as u64 > Self::MAX_OUTPUT_BYTES {
                            Self::kill_and_reap(&mut child);
                            return None;
                        }
                        collected_output = Some(bytes);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(None) | Err(_) => {
                    Self::kill_and_reap(&mut child);
                    return None;
                }
            }
        }
    }

    fn decode_output(mut bytes: Vec<u8>) -> Option<String> {
        // Remove Git's one line terminator only. This retains embedded newlines
        // in unusual but valid filesystem paths instead of splitting output.
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
        }
        String::from_utf8(bytes).ok()
    }

    fn kill_and_reap(child: &mut Child) {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::GitCliContextProvider;
    use crate::domain::git_context::{GitBranch, GitContextProvider};
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> Self {
            let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "thaa-git-context-{}-{id}-{name}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create fixture directory");
            Self(path)
        }

        fn repo(&self, relative: impl AsRef<Path>) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(&path).expect("create repository directory");
            git(&path, &["init", "--quiet"]);
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn git(directory: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_AUTHOR_NAME", "Thaa test")
            .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_NAME", "Thaa test")
            .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
            .status()
            .expect("run git fixture command");
        assert!(status.success(), "git fixture command failed: {args:?}");
    }

    fn provider() -> GitCliContextProvider {
        GitCliContextProvider::default()
    }

    struct SentinelCleanup(PathBuf);

    impl Drop for SentinelCleanup {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn resolves_root_and_current_branch_from_nested_directory() {
        let fixture = Fixture::new("nested");
        let repository = fixture.repo("repo");
        git(
            &repository,
            &["symbolic-ref", "HEAD", "refs/heads/feature/context"],
        );
        let working_directory = repository.join("packages/app/src");
        fs::create_dir_all(&working_directory).expect("create nested working directory");

        let context = provider()
            .context_for(&working_directory)
            .expect("repository context");

        assert_eq!(
            context.repository_root,
            fs::canonicalize(repository).unwrap()
        );
        assert_eq!(context.branch, GitBranch::Named("feature/context".into()));
    }

    #[test]
    fn recognizes_detached_head() {
        let fixture = Fixture::new("detached");
        let repository = fixture.repo("repo");
        fs::write(repository.join("fixture.txt"), "fixture").expect("write fixture file");
        git(&repository, &["add", "fixture.txt"]);
        git(
            &repository,
            &[
                "-c",
                "user.name=Thaa test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "fixture",
            ],
        );
        git(&repository, &["checkout", "--quiet", "--detach", "HEAD"]);

        let context = provider()
            .context_for(&repository)
            .expect("repository context");
        assert_eq!(context.branch, GitBranch::DetachedHead);
    }

    #[test]
    fn uses_nearest_nested_repository_root() {
        let fixture = Fixture::new("nested-repository");
        let outer = fixture.repo("outer");
        let inner = outer.join("vendor/inner");
        fs::create_dir_all(&inner).expect("create inner repository");
        git(&inner, &["init", "--quiet"]);

        let context = provider()
            .context_for(&inner)
            .expect("inner repository context");
        assert_eq!(context.repository_root, fs::canonicalize(inner).unwrap());
    }

    #[test]
    fn supports_spaces_and_unicode_without_shell_interpolation() {
        let fixture = Fixture::new("unicode");
        let sentinel = std::env::current_dir()
            .expect("current directory")
            .join(format!("thaa-git-context-shell-{}", std::process::id()));
        let _sentinel_cleanup = SentinelCleanup(sentinel.clone());
        let repository = fixture.repo(format!(
            "repo space-雪-$(touch {})",
            sentinel
                .file_name()
                .expect("sentinel filename")
                .to_string_lossy()
        ));
        let context = provider()
            .context_for(&repository)
            .expect("repository context");

        assert_eq!(
            context.repository_root,
            fs::canonicalize(repository).unwrap()
        );
        assert!(!sentinel.exists());
    }

    #[test]
    fn preserves_newlines_in_repository_paths() {
        let fixture = Fixture::new("line\nbreak");
        let repository = fixture.repo("repo\nroot");
        let context = provider()
            .context_for(&repository)
            .expect("repository context");
        assert_eq!(
            context.repository_root,
            fs::canonicalize(repository).unwrap()
        );
    }

    #[test]
    fn resolves_linked_worktrees_with_git_metadata_file() {
        let fixture = Fixture::new("worktree");
        let repository = fixture.repo("main");
        fs::write(repository.join("fixture.txt"), "fixture").expect("write fixture file");
        git(&repository, &["add", "fixture.txt"]);
        git(
            &repository,
            &[
                "-c",
                "user.name=Thaa test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "fixture",
            ],
        );
        let worktree = fixture.0.join("linked checkout");
        let status = Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["worktree", "add", "--quiet", "-b", "feature/worktree"])
            .arg(&worktree)
            .arg("HEAD")
            .status()
            .expect("create linked worktree");
        assert!(status.success());
        assert!(worktree.join(".git").is_file());

        let context = provider()
            .context_for(&worktree)
            .expect("worktree repository context");

        assert_eq!(context.repository_root, fs::canonicalize(worktree).unwrap());
        assert_eq!(context.branch, GitBranch::Named("feature/worktree".into()));
    }

    #[test]
    fn non_repository_and_missing_working_directory_have_no_context() {
        let fixture = Fixture::new("missing");
        assert_eq!(provider().context_for(&fixture.0), None);
        assert_eq!(provider().context_for(&fixture.0.join("missing")), None);
    }

    #[test]
    fn unavailable_git_executable_returns_no_context() {
        let fixture = Fixture::new("missing-git");
        let repository = fixture.repo("repo");
        let adapter = GitCliContextProvider {
            executable: OsString::from("/path/that/does/not/exist/thaa-git"),
        };
        assert_eq!(adapter.context_for(&repository), None);
    }
}
