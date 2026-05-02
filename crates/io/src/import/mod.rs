use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use maelstrom_core::hash::hash_file;
use time::OffsetDateTime;
use time::format_description;

use crate::catalog::catalog::Catalog;
use crate::image_files::helpers::FolderScanResult;
use crate::metadata::metadata::Metadata;

#[derive(Debug, Clone, Copy)]
pub enum ImportStrategy {
    DefaultByDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportDecision {
    Import,
    SkipAlreadyImported,
    Error,
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub source_path: PathBuf,
    pub dest_path: PathBuf,
    pub hash: String,
    pub capture_date_raw: String,
    pub decision: ImportDecision,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportPlan {
    pub root: PathBuf,
    pub items: Vec<ImportItem>,
    pub summary: ImportSummary,
}

#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    pub total: usize,
    pub to_import: usize,
    pub skipped: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ImportExecutionReport {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
}

struct TempItem {
    source_path: PathBuf,
    hash: String,
    capture_date_raw: String,
    sort_key: Option<String>,
    date_folder: Option<String>,
    decision: ImportDecision,
    error: Option<String>,
}

pub fn create_import_plan(
    root: impl AsRef<Path>,
    scan_result: &FolderScanResult,
    existing_hashes: &HashSet<String>,
    strategy: ImportStrategy,
) -> ImportPlan {
    let root = root.as_ref().to_path_buf();
    let mut items = Vec::with_capacity(scan_result.all_image_paths.len());

    for source_path in &scan_result.all_image_paths {
        let mut decision = ImportDecision::Import;
        let mut error = None;

        let hash = match hash_file(&source_path.to_path_buf()) {
            Ok(value) => value,
            Err(err) => {
                decision = ImportDecision::Error;
                error = Some(format!("Failed to hash {:?}: {}", source_path, err));
                String::new()
            }
        };

        if decision == ImportDecision::Import && existing_hashes.contains(&hash) {
            decision = ImportDecision::SkipAlreadyImported;
        }

        let capture_date_raw = read_capture_date(source_path).unwrap_or_default();
        let (sort_key, date_folder) = capture_date_parts(&capture_date_raw);

        if decision != ImportDecision::Error && date_folder.is_none() {
            decision = ImportDecision::Error;
            error = Some(format!(
                "Failed to determine capture date for {:?}",
                source_path
            ));
        }

        items.push(TempItem {
            source_path: source_path.to_path_buf(),
            hash,
            capture_date_raw,
            sort_key,
            date_folder,
            decision,
            error,
        });
    }

    match strategy {
        ImportStrategy::DefaultByDate => {
            items.sort_by(|left, right| match (&left.sort_key, &right.sort_key) {
                (Some(left_key), Some(right_key)) => left_key.cmp(right_key),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => left.source_path.cmp(&right.source_path),
            })
        }
    }

    let mut used_destinations = HashSet::new();
    let mut final_items = Vec::with_capacity(items.len());

    for item in items {
        let dest_path = if let Some(date_folder) = item.date_folder.as_ref() {
            let (year, ymd) = split_date_key(date_folder);
            let filename = item
                .source_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "image".to_string());
            let base = root.join(year).join(ymd).join(filename);
            resolve_collision(base, &mut used_destinations)
        } else {
            root.clone()
        };

        final_items.push(ImportItem {
            source_path: item.source_path,
            dest_path,
            hash: item.hash,
            capture_date_raw: item.capture_date_raw,
            decision: item.decision,
            error: item.error,
        });
    }

    let summary = summarize_import_plan(&final_items);

    ImportPlan {
        root,
        items: final_items,
        summary,
    }
}

pub async fn execute_import_plan(plan: ImportPlan, catalog: &Catalog) -> ImportExecutionReport {
    let mut report = ImportExecutionReport::default();

    for item in plan.items {
        match item.decision {
            ImportDecision::Import => {
                if item.hash.is_empty() {
                    report
                        .errors
                        .push(format!("Missing hash for {:?}", item.source_path));
                    continue;
                }

                let Some(parent) = item.dest_path.parent() else {
                    report.errors.push(format!(
                        "Invalid destination path for {:?}",
                        item.source_path
                    ));
                    continue;
                };

                if let Err(err) = fs::create_dir_all(parent) {
                    report
                        .errors
                        .push(format!("Failed to create {:?}: {}", parent, err));
                    continue;
                }

                if let Err(err) = fs::copy(&item.source_path, &item.dest_path) {
                    report.errors.push(format!(
                        "Failed to copy {:?} to {:?}: {}",
                        item.source_path, item.dest_path, err
                    ));
                    continue;
                }

                if let Err(err) = catalog.add_image(&item.hash, &item.dest_path).await {
                    report
                        .errors
                        .push(format!("Failed to catalog {:?}: {}", item.dest_path, err));
                    continue;
                }

                report.imported_count += 1;
            }
            ImportDecision::SkipAlreadyImported => {
                report.skipped_count += 1;
            }
            ImportDecision::Error => {
                report.errors.push(
                    item.error
                        .unwrap_or_else(|| format!("Import error for {:?}", item.source_path)),
                );
            }
        }
    }

    report
}

fn summarize_import_plan(items: &[ImportItem]) -> ImportSummary {
    let mut summary = ImportSummary {
        total: items.len(),
        ..Default::default()
    };

    for item in items {
        match item.decision {
            ImportDecision::Import => summary.to_import += 1,
            ImportDecision::SkipAlreadyImported => summary.skipped += 1,
            ImportDecision::Error => summary.errors += 1,
        }
    }

    summary
}

fn read_capture_date(path: &Path) -> Option<String> {
    if let Ok(metadata) = Metadata::read_exif(path)
        && let Some(value) = metadata.capture_date
    {
        return Some(value);
    }

    let meta = fs::metadata(path).ok()?;
    let time = meta.created().ok().or_else(|| meta.modified().ok())?;
    format_system_time(time)
}

fn format_system_time(time: SystemTime) -> Option<String> {
    let format = format_description::parse("[year]:[month]:[day] [hour]:[minute]:[second]").ok()?;
    OffsetDateTime::from(time).format(&format).ok()
}

fn capture_date_parts(raw: &str) -> (Option<String>, Option<String>) {
    if raw.is_empty() {
        return (None, None);
    }

    let date_part = match raw.split_whitespace().next() {
        Some(value) => value,
        None => return (None, None),
    };

    if date_part.len() < 10 {
        return (Some(raw.to_string()), None);
    }

    let date_folder = date_part.replace(':', "-");
    (Some(raw.to_string()), Some(date_folder))
}

fn split_date_key(date_key: &str) -> (String, String) {
    let year = date_key.get(0..4).unwrap_or("0000").to_string();
    (year, date_key.to_string())
}

fn resolve_collision(base: PathBuf, used: &mut HashSet<PathBuf>) -> PathBuf {
    if !base.exists() && !used.contains(&base) {
        used.insert(base.clone());
        return base;
    }

    let stem = base
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("image");
    let ext = base.extension().and_then(|value| value.to_str());
    let parent = base.parent().unwrap_or_else(|| Path::new(""));

    for index in 1..=9999 {
        let file_name = match ext {
            Some(ext) => format!("{}-{}.{}", stem, index, ext),
            None => format!("{}-{}", stem, index),
        };
        let candidate = parent.join(file_name);
        if !candidate.exists() && !used.contains(&candidate) {
            used.insert(candidate.clone());
            return candidate;
        }
    }

    base
}
