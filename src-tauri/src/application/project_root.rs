//! Best-effort project-root detection from a process working directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Detects the nearest supported project marker in `working_directory` or
/// one of its ancestors. Filesystem errors intentionally collapse to `None`:
/// project context is optional and must not fail runtime inspection.
pub(crate) fn detect_project_root(working_directory: &Path) -> Option<PathBuf> {
    let canonical_directory = fs::canonicalize(working_directory).ok()?;
    if !canonical_directory.is_dir() {
        return None;
    }

    let mut current = canonical_directory.as_path();
    loop {
        // Use the canonicalized start path here as well as for traversal. On
        // Windows, the original path may be a DOS path while canonicalize
        // returns an extended-length path; comparing volume names derived
        // from those two spellings can falsely report different volumes.
        match crate::platform::same_filesystem(&canonical_directory, current) {
            Ok(true) => {}
            Ok(false) | Err(_) => return None,
        }

        match contains_project_marker(current) {
            Ok(true) => return Some(current.to_path_buf()),
            Ok(false) => {}
            Err(_) => return None,
        }

        let parent = current.parent()?;
        if parent == current {
            return None;
        }
        current = parent;
    }
}

fn contains_project_marker(directory: &Path) -> io::Result<bool> {
    const FILE_MARKERS: &[&str] = &[
        "package.json",
        "Cargo.toml",
        "go.mod",
        "pyproject.toml",
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
        "composer.json",
        "pnpm-workspace.yaml",
    ];

    for marker in FILE_MARKERS {
        match fs::metadata(directory.join(marker)) {
            Ok(metadata) if metadata.is_file() => return Ok(true),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    match fs::metadata(directory.join(".git")) {
        Ok(metadata) if metadata.is_file() || metadata.is_dir() => return Ok(true),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    has_dotnet_marker(directory)
}

/// Checks direct children only; it does not recurse into the directory tree.
fn has_dotnet_marker(directory: &Path) -> io::Result<bool> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let extension = Path::new(&file_name)
            .extension()
            .and_then(|value| value.to_str());
        let supported_extension = if cfg!(windows) {
            extension.is_some_and(|value| {
                value.eq_ignore_ascii_case("sln") || value.eq_ignore_ascii_case("csproj")
            })
        } else {
            matches!(extension, Some("sln" | "csproj"))
        };
        if supported_extension && entry.metadata()?.is_file() {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::detect_project_root;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("thaa-project-root-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).expect("create fixture root");
            Self(path)
        }

        fn dir(&self, relative: impl AsRef<Path>) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(&path).expect("create fixture directory");
            path
        }

        fn file(&self, relative: impl AsRef<Path>) {
            let path = self.0.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create marker parent");
            }
            fs::write(path, b"marker contents are never read").expect("create marker");
        }

        fn canonical(&self, relative: impl AsRef<Path>) -> PathBuf {
            fs::canonicalize(self.0.join(relative)).expect("canonicalize fixture path")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn marker_in_starting_directory_is_selected() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("project/src");
        fixture.file("project/Cargo.toml");

        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("project"))
        );
    }

    #[test]
    fn searches_immediate_and_distant_ancestors() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("one/two/three/four");
        fixture.file("one/two/package.json");

        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("one/two"))
        );

        fixture.file("one/two/three/go.mod");
        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("one/two/three"))
        );
    }

    #[test]
    fn nested_project_marker_wins_over_a_distant_git_root() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("repo/app/src");
        fixture.dir("repo/.git");
        fixture.file("repo/app/package.json");

        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("repo/app"))
        );
    }

    #[test]
    fn nearest_package_in_workspace_wins_over_workspace_marker() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("repo/apps/desktop/src");
        fixture.file("repo/.git/HEAD");
        fixture.file("repo/pnpm-workspace.yaml");
        fixture.file("repo/package.json");
        fixture.file("repo/apps/desktop/package.json");

        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("repo/apps/desktop"))
        );
    }

    #[test]
    fn workspace_marker_is_root_when_no_nearer_marker_exists() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("repo/apps/desktop/src");
        fixture.file("repo/pnpm-workspace.yaml");

        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("repo"))
        );
    }

    #[test]
    fn every_supported_marker_family_is_recognized() {
        let markers = [
            "package.json",
            "pnpm-workspace.yaml",
            "Cargo.toml",
            "go.mod",
            "pyproject.toml",
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
            "composer.json",
            "Sample.sln",
            "Sample.csproj",
        ];

        for marker in markers {
            let fixture = Fixture::new();
            let working_directory = fixture.dir("root/nested");
            fixture.file(Path::new("root").join(marker));
            assert_eq!(
                detect_project_root(&working_directory),
                Some(fixture.canonical("root")),
                "marker {marker}"
            );
        }
    }

    #[test]
    fn git_marker_may_be_a_file_or_directory() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("repo/src");
        fixture.file("repo/.git");
        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("repo"))
        );

        fs::remove_file(fixture.0.join("repo/.git")).expect("remove git file marker");
        fixture.dir("repo/.git");
        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("repo"))
        );
    }

    #[test]
    fn multiple_markers_in_one_directory_produce_one_root() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("project/src");
        fixture.file("project/package.json");
        fixture.file("project/Cargo.toml");
        fixture.dir("project/.git");

        assert_eq!(
            detect_project_root(&working_directory),
            Some(fixture.canonical("project"))
        );
    }

    #[test]
    fn generic_files_and_nested_dotnet_markers_do_not_create_roots() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("workspace/app/src");
        fixture.file("workspace/README.md");
        fixture.file("workspace/Makefile");
        fixture.file("workspace/requirements.txt");
        fixture.file("workspace/app/nested/Sample.csproj");

        assert_eq!(detect_project_root(&working_directory), None);
    }

    #[test]
    fn no_marker_returns_none_after_reaching_filesystem_root() {
        let fixture = Fixture::new();
        let working_directory = fixture.dir("unmarked/nested");
        assert_eq!(detect_project_root(&working_directory), None);
    }

    #[test]
    fn unavailable_nonexistent_and_non_directory_paths_return_none() {
        let fixture = Fixture::new();
        let regular_file = fixture.0.join("file");
        fs::write(&regular_file, b"not a directory").expect("create regular file");

        assert_eq!(detect_project_root(Path::new("")), None);
        assert_eq!(detect_project_root(&fixture.0.join("missing")), None);
        assert_eq!(detect_project_root(&regular_file), None);
    }

    #[cfg(unix)]
    #[test]
    fn canonicalizes_a_symlinked_working_directory() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        let working_directory = fixture.dir("real/project/src");
        fixture.file("real/project/package.json");
        symlink(&working_directory, fixture.0.join("alias")).expect("create cwd symlink");

        assert_eq!(
            detect_project_root(&fixture.0.join("alias")),
            Some(fixture.canonical("real/project"))
        );
    }
}
