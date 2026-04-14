pub mod autofill;
pub mod detectors;
pub mod types;

use std::path::Path;
use std::time::Instant;
use types::{ScanProgress, ScanReport};
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".cache",
    ".terraform",
    ".tofu",
    "vendor",
];

const MAX_DEPTH: usize = 4;

pub fn run_scan<F>(repo_path: &Path, on_progress: F) -> Result<ScanReport, String>
where
    F: Fn(ScanProgress),
{
    if !repo_path.is_dir() {
        return Err(format!("Not a directory: {}", repo_path.display()));
    }

    let start = Instant::now();

    // Phase 1: Walk directory to count files
    on_progress(ScanProgress {
        phase: "walking".to_string(),
        files_checked: 0,
        detections_so_far: 0,
    });

    let mut files_scanned: u64 = 0;
    let walker = WalkDir::new(repo_path)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref())
        });

    for entry in walker {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                files_scanned += 1;
                if files_scanned % 100 == 0 {
                    on_progress(ScanProgress {
                        phase: "walking".to_string(),
                        files_checked: files_scanned,
                        detections_so_far: 0,
                    });
                }
            }
        }
    }

    // Phase 2: Run detectors
    on_progress(ScanProgress {
        phase: "detecting".to_string(),
        files_checked: files_scanned,
        detections_so_far: 0,
    });

    let detections = detectors::detect_all(repo_path);
    let inferred_stack = detectors::infer_stack(&detections);

    on_progress(ScanProgress {
        phase: "complete".to_string(),
        files_checked: files_scanned,
        detections_so_far: detections.len() as u64,
    });

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(ScanReport {
        detections,
        files_scanned,
        duration_ms,
        inferred_stack,
    })
}
