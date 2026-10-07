//! Conservative runtime-family classification from observed process names.

use std::ffi::OsStr;

/// A runtime family inferred from an observed process name.
///
/// This value is presentation metadata only. It is not process identity,
/// project-language evidence, or a security assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeKind {
    NodeJs,
    Python,
    Java,
    Ruby,
    Php,
}

/// Classifies only exact, known runtime-host executable names.
///
/// Non-Unicode names are left unclassified rather than lossily converted.
pub(crate) fn classify_runtime_name(name: &OsStr) -> Option<RuntimeKind> {
    let name = name.to_str()?;
    let name = strip_exe_suffix(name);

    if matches_name(name, &["node", "nodejs"]) {
        Some(RuntimeKind::NodeJs)
    } else if is_python_name(name) {
        Some(RuntimeKind::Python)
    } else if matches_name(name, &["java", "javaw"]) {
        Some(RuntimeKind::Java)
    } else if matches_name(name, &["ruby", "rubyw"]) {
        Some(RuntimeKind::Ruby)
    } else if matches_name(name, &["php", "php-cgi"]) {
        Some(RuntimeKind::Php)
    } else {
        None
    }
}

fn strip_exe_suffix(name: &str) -> &str {
    if name.len() >= 4
        && name
            .get(name.len() - 4..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case(".exe"))
    {
        &name[..name.len() - 4]
    } else {
        name
    }
}

fn matches_name(name: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

fn is_python_name(name: &str) -> bool {
    if matches_name(name, &["python", "python2", "python3"]) {
        return true;
    }

    ["python2.", "python3."].iter().any(|prefix| {
        name.get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
            && name[prefix.len()..]
                .split('.')
                .all(|segment| !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit()))
    })
}

#[cfg(test)]
mod tests {
    use super::{classify_runtime_name, RuntimeKind};
    use std::ffi::OsStr;

    #[test]
    fn recognizes_supported_runtime_hosts_and_aliases() {
        let cases = [
            ("node", RuntimeKind::NodeJs),
            ("nodejs", RuntimeKind::NodeJs),
            ("python", RuntimeKind::Python),
            ("python2", RuntimeKind::Python),
            ("python3", RuntimeKind::Python),
            ("python3.13", RuntimeKind::Python),
            ("python3.13.1", RuntimeKind::Python),
            ("python2.7", RuntimeKind::Python),
            ("java", RuntimeKind::Java),
            ("javaw", RuntimeKind::Java),
            ("ruby", RuntimeKind::Ruby),
            ("rubyw", RuntimeKind::Ruby),
            ("php", RuntimeKind::Php),
            ("php-cgi", RuntimeKind::Php),
        ];

        for (name, expected) in cases {
            assert_eq!(classify_runtime_name(OsStr::new(name)), Some(expected));
        }
    }

    #[test]
    fn matches_ascii_case_and_windows_executable_suffix() {
        let cases = [
            ("NODE.EXE", RuntimeKind::NodeJs),
            ("NodeJs", RuntimeKind::NodeJs),
            ("PYTHON3.13.EXE", RuntimeKind::Python),
            ("JAVA.EXE", RuntimeKind::Java),
            ("RubyW.exe", RuntimeKind::Ruby),
            ("PHP-CGI.EXE", RuntimeKind::Php),
        ];

        for (name, expected) in cases {
            assert_eq!(classify_runtime_name(OsStr::new(name)), Some(expected));
        }
    }

    #[test]
    fn rejects_near_matches_wrappers_and_unrelated_processes() {
        let cases = [
            "node-helper",
            "javascript-tool",
            "npm",
            "npx",
            "my-python-helper",
            "python3.helper",
            "python3.",
            "java-wrapper",
            "gradle",
            "ruby-shim",
            "php-worker",
            "bash",
            "chrome",
            "",
        ];

        for name in cases {
            assert_eq!(classify_runtime_name(OsStr::new(name)), None, "{name}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn leaves_non_unicode_process_names_unknown() {
        use std::os::unix::ffi::OsStrExt;

        assert_eq!(classify_runtime_name(OsStr::from_bytes(b"node\xff")), None);
    }
}
