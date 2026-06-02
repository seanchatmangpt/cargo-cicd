use std::process::Command;

pub struct ChangedFileDetector;

impl ChangedFileDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn changed_rs_files(base: &str) -> Vec<String> {
        let output = Command::new("git")
            .args(["diff", "--name-only", base, "--", "*.rs"])
            .output()
            .unwrap_or_else(|_| std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn is_test_file(path: &str) -> bool {
        path.contains("/tests/") || path.ends_with("_test.rs") || path.ends_with("_tests.rs")
    }

    pub fn is_trybuild_fixture(path: &str) -> bool {
        path.contains("/tests/")
            && (path.ends_with(".rs") || path.ends_with(".stderr") || path.ends_with(".stdout"))
            && (path.contains("compile_fail") || path.contains("trybuild") || path.contains("ui/"))
    }
}

impl Default for ChangedFileDetector {
    fn default() -> Self {
        Self::new()
    }
}
