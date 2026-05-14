use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use chrono::Local;
use walkdir::WalkDir;

use crate::config::Config;
use crate::model::*;

const COMMIT_DELIM: &str = "COMMIT_DELIM";

fn type_label(t: &str) -> &str {
    match t {
        "feat" => "新功能",
        "fix" => "錯誤修復",
        "docs" => "文件",
        "refactor" => "重構",
        "test" => "測試",
        "chore" => "維護",
        "ci" => "CI/CD",
        "build" => "建構",
        "style" => "格式",
        "perf" => "效能",
        _ => "其他",
    }
}

fn parse_conventional_commit(subject: &str) -> (String, String) {
    // Match ^(\w+)(\(.*?\))?(!)?:
    let bytes = subject.as_bytes();
    let mut i = 0;

    // Parse type: sequence of word chars
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    if i == 0 {
        return ("other".to_string(), String::new());
    }
    let commit_type = &subject[..i];

    // Optional scope in parens
    let mut scope = String::new();
    let mut j = i;
    if j < bytes.len()
        && bytes[j] == b'('
        && let Some(close) = subject[j..].find(')')
    {
        scope = subject[j + 1..j + close].to_string();
        j += close + 1;
    }

    // Optional ! before :
    if j < bytes.len() && bytes[j] == b'!' {
        j += 1;
    }

    // Must have :
    if j < bytes.len() && bytes[j] == b':' {
        (commit_type.to_lowercase(), scope)
    } else {
        ("other".to_string(), String::new())
    }
}

fn collect_proposals(repo_path: &str, project_name: &str) -> Vec<ProposalEntry> {
    let archive_path = Path::new(repo_path).join("openspec/changes/archive");
    if !archive_path.exists() {
        return Vec::new();
    }

    let mut proposals = Vec::new();

    let entries = match std::fs::read_dir(&archive_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Parse YYYY-MM-DD-slug
        if dir_name.len() < 11 || dir_name.as_bytes()[10] != b'-' {
            continue;
        }
        let date = &dir_name[..10];
        // Validate date format loosely
        if date.len() != 10 || date.as_bytes()[4] != b'-' || date.as_bytes()[7] != b'-' {
            continue;
        }
        let slug = &dir_name[11..];

        let dir_path = entry.path();

        // Count tasks from tasks.md
        let task_count = std::fs::read_to_string(dir_path.join("tasks.md"))
            .map(|content| content.lines().filter(|l| l.contains("- [x]")).count())
            .unwrap_or(0);

        // Read description from .openspec.yaml
        let description = std::fs::read_to_string(dir_path.join(".openspec.yaml"))
            .map(|content| {
                content
                    .lines()
                    .find_map(|l| {
                        let trimmed = l.trim();
                        trimmed
                            .strip_prefix("description:")
                            .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        proposals.push(ProposalEntry {
            slug: slug.to_string(),
            date: date.to_string(),
            project: project_name.to_string(),
            description,
            task_count,
        });
    }

    proposals.sort_by(|a, b| b.date.cmp(&a.date));
    proposals
}

fn apply_filters(
    commits: &[CommitEntry],
    proposals: &[ProposalEntry],
    config: &Config,
) -> (Vec<CommitEntry>, Vec<ProposalEntry>) {
    let filters = config.filters();

    let filtered_commits: Vec<CommitEntry> = commits
        .iter()
        .filter(|c| {
            if let Some(ref author) = filters.author
                && !c.subject.is_empty()
                && !c.project.is_empty()
            {
                // We need to re-check author from the git log.
                // Actually, author info is not in CommitEntry. We need to filter during collection.
                // For now, this filter is applied during collection in a wrapper.
                // But we stored author filtering needs to happen at collection time.
                // Let's keep this as a pass-through; author filtering happens in collect().
                let _ = author;
            }
            if let Some(ref hashes) = filters.exclude_hashes
                && hashes.iter().any(|h| h == &c.hash)
            {
                return false;
            }
            if let Some(ref since) = filters.since
                && c.date < *since
            {
                return false;
            }
            if let Some(ref until) = filters.until
                && c.date > *until
            {
                return false;
            }
            if let Some(ref types) = filters.types
                && !types.contains(&c.commit_type)
            {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    let filtered_proposals: Vec<ProposalEntry> = proposals
        .iter()
        .filter(|p| {
            if let Some(ref since) = filters.since
                && p.date < *since
            {
                return false;
            }
            if let Some(ref until) = filters.until
                && p.date > *until
            {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    (filtered_commits, filtered_proposals)
}

fn parse_shortstat(line: &str) -> (usize, usize) {
    let mut insertions = 0;
    let mut deletions = 0;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return (0, 0);
    }
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.contains("insertion") {
            if let Some(n) = part.split_whitespace().next() {
                insertions = n.parse().unwrap_or(0);
            }
        } else if part.contains("deletion")
            && let Some(n) = part.split_whitespace().next()
        {
            deletions = n.parse().unwrap_or(0);
        }
    }
    (insertions, deletions)
}

fn collect_git_commits_filtered(
    repo_path: &str,
    project_name: &str,
    author_filter: Option<&str>,
    branch: Option<&str>,
) -> Result<Vec<CommitEntry>> {
    let path = Path::new(repo_path);
    if !path.exists() || !path.join(".git").exists() {
        eprintln!("Warning: skipping '{}' — not a git repository", repo_path);
        return Ok(Vec::new());
    }

    let mut args = vec!["-C".to_string(), repo_path.to_string(), "log".to_string()];

    if let Some(branch_name) = branch {
        args.push(branch_name.to_string());
    } else {
        args.push("--all".to_string());
    }

    args.push(format!("--format={}%H|||%aN|||%aI|||%s", COMMIT_DELIM));
    args.push("--shortstat".to_string());

    if let Some(author) = author_filter {
        args.push(format!("--author={}", author));
    }

    let output = Command::new("git")
        .env("LC_ALL", "C")
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let ref_desc = branch.unwrap_or("--all");
        eprintln!(
            "Warning: git log failed for '{}' (ref: {}): {}",
            project_name, ref_desc, stderr
        );
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    let mut current: Option<CommitEntry> = None;

    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix(COMMIT_DELIM) {
            if let Some(c) = current.take() {
                commits.push(c);
            }
            let parts: Vec<&str> = rest.splitn(4, "|||").collect();
            if parts.len() == 4 {
                let (mut commit_type, scope) = parse_conventional_commit(parts[3]);
                if project_name.ends_with(".wiki") {
                    commit_type = "docs".to_string();
                }
                current = Some(CommitEntry {
                    hash: parts[0].to_string(),
                    date: parts[2][..10].to_string(),
                    commit_type,
                    scope,
                    subject: parts[3].to_string(),
                    project: project_name.to_string(),
                    insertions: 0,
                    deletions: 0,
                });
            }
        } else if let Some(ref mut c) = current {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let (ins, del) = parse_shortstat(trimmed);
                c.insertions = ins;
                c.deletions = del;
            }
        }
    }
    if let Some(c) = current.take() {
        commits.push(c);
    }

    Ok(commits)
}

fn build_timeline(commits: &[CommitEntry]) -> Vec<TimelineEntry> {
    use chrono::Datelike;

    let dates: Vec<(chrono::NaiveDate, usize)> = commits
        .iter()
        .filter_map(|c| {
            chrono::NaiveDate::parse_from_str(&c.date, "%Y-%m-%d")
                .ok()
                .map(|d| (d, c.insertions + c.deletions))
        })
        .collect();

    if dates.is_empty() {
        return vec![];
    }

    let min_date = dates.iter().map(|(d, _)| *d).min().unwrap();
    let max_date = dates.iter().map(|(d, _)| *d).max().unwrap();

    // Five-level granularity cascade: escalate when distinct bucket count > 14
    #[derive(Clone, Copy)]
    enum Granularity {
        Daily,
        Weekly,
        Monthly,
        Quarterly,
        Yearly,
    }

    let day_count = (max_date - min_date).num_days() + 1; // inclusive

    let granularity = if day_count <= 14 {
        Granularity::Daily
    } else {
        // Count distinct ISO weeks (early exit once > 14)
        let mut week_set = std::collections::HashSet::new();
        let mut d = min_date;
        while d <= max_date {
            week_set.insert(d.format("%G-W%V").to_string());
            if week_set.len() > 14 {
                break;
            }
            d += chrono::Duration::days(1);
        }
        if week_set.len() <= 14 {
            Granularity::Weekly
        } else {
            let month_count = (max_date.year() - min_date.year()) * 12 + max_date.month() as i32
                - min_date.month() as i32
                + 1;
            if month_count <= 14 {
                Granularity::Monthly
            } else {
                let min_q = (min_date.month() as i32 - 1) / 3 + 1;
                let max_q = (max_date.month() as i32 - 1) / 3 + 1;
                let quarter_count = (max_date.year() - min_date.year()) * 4 + max_q - min_q + 1;
                if quarter_count <= 14 {
                    Granularity::Quarterly
                } else {
                    Granularity::Yearly
                }
            }
        }
    };

    // Format a date into a bucket label based on selected granularity
    let format_label = |date: &chrono::NaiveDate| -> String {
        match granularity {
            Granularity::Daily => date.format("%Y-%m-%d").to_string(),
            Granularity::Weekly => date.format("%G-W%V").to_string(),
            Granularity::Monthly => format!("{:04}-{:02}", date.year(), date.month()),
            Granularity::Quarterly => {
                let q = (date.month() - 1) / 3 + 1;
                format!("{}-Q{}", date.year(), q)
            }
            Granularity::Yearly => format!("{}", date.year()),
        }
    };

    // Aggregate commit lines into buckets
    let mut bucket_lines: HashMap<String, usize> = HashMap::new();
    for (date, lines) in &dates {
        let label = format_label(date);
        *bucket_lines.entry(label).or_insert(0) += lines;
    }

    // Generate contiguous bucket labels from min_date to max_date
    let all_labels: Vec<String> = match granularity {
        Granularity::Daily => {
            let mut labels = Vec::new();
            let mut d = min_date;
            while d <= max_date {
                labels.push(d.format("%Y-%m-%d").to_string());
                d += chrono::Duration::days(1);
            }
            labels
        }
        Granularity::Weekly => {
            let mut labels = Vec::new();
            let mut d = min_date;
            let mut last_label = String::new();
            while d <= max_date {
                let label = d.format("%G-W%V").to_string();
                if label != last_label {
                    labels.push(label.clone());
                    last_label = label;
                }
                d += chrono::Duration::days(1);
            }
            labels
        }
        Granularity::Monthly => {
            let mut labels = Vec::new();
            let (mut y, mut m) = (min_date.year(), min_date.month());
            let (end_y, end_m) = (max_date.year(), max_date.month());
            loop {
                labels.push(format!("{:04}-{:02}", y, m));
                if y == end_y && m == end_m {
                    break;
                }
                m += 1;
                if m > 12 {
                    m = 1;
                    y += 1;
                }
            }
            labels
        }
        Granularity::Quarterly => {
            let mut labels = Vec::new();
            let (mut y, mut q) = (min_date.year(), (min_date.month() as i32 - 1) / 3 + 1);
            let (end_y, end_q) = (max_date.year(), (max_date.month() as i32 - 1) / 3 + 1);
            loop {
                labels.push(format!("{}-Q{}", y, q));
                if y == end_y && q == end_q {
                    break;
                }
                q += 1;
                if q > 4 {
                    q = 1;
                    y += 1;
                }
            }
            labels
        }
        Granularity::Yearly => (min_date.year()..=max_date.year())
            .map(|y| format!("{}", y))
            .collect(),
    };

    let max_lines = bucket_lines.values().copied().max().unwrap_or(0);

    all_labels
        .into_iter()
        .map(|label| {
            let lines = *bucket_lines.get(&label).unwrap_or(&0);
            TimelineEntry {
                label,
                lines,
                height: if max_lines == 0 {
                    0.0
                } else {
                    (lines as f64 / max_lines as f64) * 100.0
                },
            }
        })
        .collect()
}

fn build_type_breakdown(commits: &[CommitEntry]) -> Vec<TypeBreakdown> {
    let mut label_lines: HashMap<String, (String, usize)> = HashMap::new();
    for c in commits {
        let label = type_label(&c.commit_type).to_string();
        let canonical_type = match label.as_str() {
            "其他" => "other",
            _ => &c.commit_type,
        };
        let entry = label_lines
            .entry(label.clone())
            .or_insert_with(|| (canonical_type.to_string(), 0));
        entry.1 += c.insertions + c.deletions;
    }

    let total_lines: usize = commits.iter().map(|c| c.insertions + c.deletions).sum();
    let mut breakdown: Vec<TypeBreakdown> = label_lines
        .into_iter()
        .map(|(label, (commit_type, lines))| TypeBreakdown {
            label,
            commit_type,
            lines,
            percentage: if total_lines == 0 {
                0.0
            } else {
                (lines as f64 / total_lines as f64) * 100.0
            },
        })
        .collect();

    breakdown.sort_by_key(|b| std::cmp::Reverse(b.lines));
    breakdown
}

fn build_project_data(
    project_name: &str,
    description: &str,
    url: Option<&str>,
    commits: &[CommitEntry],
    proposals: &[ProposalEntry],
) -> ProjectData {
    let project_commits: Vec<&CommitEntry> = commits
        .iter()
        .filter(|c| c.project == project_name)
        .collect();
    let project_proposals: Vec<&ProposalEntry> = proposals
        .iter()
        .filter(|p| p.project == project_name)
        .collect();

    let lines_added: usize = project_commits.iter().map(|c| c.insertions).sum();
    let lines_removed: usize = project_commits.iter().map(|c| c.deletions).sum();

    let mut type_lines: HashMap<String, usize> = HashMap::new();
    for c in &project_commits {
        *type_lines.entry(c.commit_type.clone()).or_insert(0) += c.insertions + c.deletions;
    }
    let total_lines: usize = project_commits
        .iter()
        .map(|c| c.insertions + c.deletions)
        .sum();
    let mut top_types: Vec<TypeBreakdown> = type_lines
        .into_iter()
        .map(|(t, lines)| TypeBreakdown {
            label: type_label(&t).to_string(),
            commit_type: t,
            lines,
            percentage: if total_lines == 0 {
                0.0
            } else {
                (lines as f64 / total_lines as f64) * 100.0
            },
        })
        .collect();
    top_types.sort_by_key(|b| std::cmp::Reverse(b.lines));
    top_types.truncate(5);

    ProjectData {
        name: project_name.to_string(),
        description: description.to_string(),
        commit_count: project_commits.len(),
        proposal_count: project_proposals.len(),
        lines_added,
        lines_removed,
        top_types,
        url: url.map(|s| s.to_string()),
    }
}

fn detect_framework(project_path: &Path) -> String {
    // Check pyproject.toml for pytest
    let pyproject = project_path.join("pyproject.toml");
    if pyproject.exists()
        && let Ok(content) = std::fs::read_to_string(&pyproject)
        && content.contains("pytest")
    {
        return "pytest".to_string();
    }
    // Check Cargo.toml
    if project_path.join("Cargo.toml").exists() {
        return "cargo test".to_string();
    }
    // Check package.json for vitest or jest
    let pkg = project_path.join("package.json");
    if pkg.exists()
        && let Ok(content) = std::fs::read_to_string(&pkg)
    {
        if content.contains("vitest") {
            return "vitest".to_string();
        }
        if content.contains("jest") {
            return "jest".to_string();
        }
    }
    // Check deno.json / deno.jsonc for vitest or plain deno test
    for deno_config in ["deno.json", "deno.jsonc"] {
        let deno_path = project_path.join(deno_config);
        if deno_path.exists()
            && let Ok(content) = std::fs::read_to_string(&deno_path)
        {
            if content.contains("vitest") {
                return "vitest".to_string();
            }
            return "deno test".to_string();
        }
    }
    "none".to_string()
}

fn is_excluded_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules" | "target" | ".venv" | "__pycache__" | ".git" | "dist" | ".tox"
    )
}

fn is_test_file(path: &Path, framework: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };
    match framework {
        "pytest" => {
            file_name.starts_with("test_") && file_name.ends_with(".py")
                || file_name.ends_with("_test.py")
        }
        "vitest" | "jest" => file_name.contains(".test.") || file_name.contains(".spec."),
        "deno test" => {
            file_name.ends_with("_test.ts")
                || file_name.ends_with("_test.js")
                || file_name.ends_with("_test.mjs")
                || file_name.contains(".test.")
        }
        "cargo test" => {
            // For Rust, test files in tests/ dir, or any .rs file — we'll check content
            if let Some(ext) = path.extension() {
                ext == "rs"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn discover_test_files(project_path: &Path, framework: &str) -> Vec<std::path::PathBuf> {
    let mut test_files = Vec::new();
    let walker = WalkDir::new(project_path).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !is_excluded_dir(&name)
        } else {
            true
        }
    });
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && is_test_file(entry.path(), framework) {
            // For Rust, only count files that contain #[test] or #[cfg(test)]
            if framework == "cargo test" {
                if let Ok(content) = std::fs::read_to_string(entry.path())
                    && (content.contains("#[test]")
                        || content.contains("#[cfg(test)]")
                        || content.contains("#[tokio::test")
                        || content.contains("#[rstest"))
                {
                    test_files.push(entry.path().to_path_buf());
                }
            } else {
                test_files.push(entry.path().to_path_buf());
            }
        }
    }
    test_files
}

fn count_test_cases(test_files: &[std::path::PathBuf], framework: &str) -> usize {
    let mut count = 0;
    for path in test_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                match framework {
                    "pytest"
                        if (trimmed.starts_with("def test_")
                            || trimmed.starts_with("async def test_")) =>
                    {
                        count += 1;
                    }
                    "vitest" | "jest"
                        if (trimmed.contains("it(")
                            || trimmed.contains("it.each(")
                            || trimmed.contains("test(")
                            || trimmed.contains("test.each("))
                            && !trimmed.starts_with("//")
                            && !trimmed.starts_with("*") =>
                    {
                        count += 1;
                    }
                    "cargo test"
                        if (trimmed.contains("#[test]")
                            || trimmed.contains("#[tokio::test")
                            || trimmed.contains("#[rstest")) =>
                    {
                        count += 1;
                    }
                    "deno test"
                        if trimmed.starts_with("Deno.test(")
                            || trimmed.starts_with("Deno.test.only(")
                            || trimmed.starts_with("Deno.test.ignore(") =>
                    {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    count
}

fn parse_cobertura_xml(path: &Path) -> Option<f64> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let pos = content.find("line-rate=\"")?;
    let start = pos + "line-rate=\"".len();
    let end = content[start..].find('"')?;
    let rate = content[start..start + end].parse::<f64>().ok()?;
    Some(rate * 100.0)
}

fn parse_istanbul_json(path: &Path) -> Option<f64> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("total")
        .and_then(|t| t.get("lines"))
        .and_then(|l| l.get("pct"))
        .and_then(|p| p.as_f64())
}

fn discover_coverage(project_path: &Path, result_path: Option<&str>) -> Option<f64> {
    // If explicit path is provided, try that first
    if let Some(rel_path) = result_path {
        let full_path = project_path.join(rel_path);
        if full_path.exists() {
            let filename = full_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename.ends_with(".xml") {
                return parse_cobertura_xml(&full_path);
            } else if filename.ends_with(".json") {
                return parse_istanbul_json(&full_path);
            }
        }
        return None;
    }

    // Auto-discovery fallback
    if let Some(pct) = parse_cobertura_xml(&project_path.join("coverage.xml")) {
        return Some(pct);
    }
    parse_istanbul_json(&project_path.join("coverage/coverage-summary.json"))
}

fn collect_test_metrics(config: &Config) -> Vec<TestMetrics> {
    // Phase 1: Spawn all coverage commands in parallel
    let mut children: Vec<(usize, std::process::Child)> = Vec::new();
    for (i, project) in config.projects.iter().enumerate() {
        if let Some(ref cmd) = project.coverage_command {
            let project_path = Path::new(&project.path);
            if !project_path.exists() {
                continue;
            }
            eprintln!("  Starting coverage: {} ({})", project.name, cmd);
            match Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(project_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::inherit())
                .spawn()
            {
                Ok(child) => children.push((i, child)),
                Err(e) => eprintln!(
                    "  Warning: failed to start coverage for '{}': {}",
                    project.name, e
                ),
            }
        }
    }

    // Phase 2: Wait for all coverage commands to finish
    for (idx, mut child) in children {
        let name = &config.projects[idx].name;
        match child.wait() {
            Ok(status) => {
                if status.success() {
                    eprintln!("  Coverage done: {}", name);
                } else {
                    eprintln!("  Warning: coverage failed for '{}'", name);
                }
            }
            Err(e) => eprintln!("  Warning: coverage wait error for '{}': {}", name, e),
        }
    }

    // Phase 3: Collect metrics (file discovery + parse coverage results)
    let mut metrics = Vec::new();
    for project in &config.projects {
        let project_path = Path::new(&project.path);
        if !project_path.exists() {
            continue;
        }
        let framework = detect_framework(project_path);
        let (test_file_count, test_case_count) = if framework == "none" {
            (0, 0)
        } else {
            let test_files = discover_test_files(project_path, &framework);
            let test_case_count = count_test_cases(&test_files, &framework);
            (test_files.len(), test_case_count)
        };
        let coverage_percent =
            discover_coverage(project_path, project.coverage_result_path.as_deref());
        metrics.push(TestMetrics {
            project: project.name.clone(),
            test_file_count,
            test_case_count,
            coverage_percent,
            framework,
        });
    }
    metrics
}

pub fn collect(config: &Config) -> Result<ShowcaseData> {
    let filters = config.filters();
    let author_filter = filters.author.as_deref();

    let mut all_commits = Vec::new();
    let mut all_proposals = Vec::new();

    for project in &config.projects {
        let commits = collect_git_commits_filtered(
            &project.path,
            &project.name,
            author_filter,
            project.branch.as_deref(),
        )?;
        all_commits.extend(commits);

        let proposals = collect_proposals(&project.path, &project.name);
        all_proposals.extend(proposals);
    }

    // Apply date and type filters
    let (all_commits, all_proposals) = apply_filters(&all_commits, &all_proposals, config);

    // Sort commits by date descending
    let mut all_commits = all_commits;
    all_commits.sort_by(|a, b| b.date.cmp(&a.date));

    let mut all_proposals = all_proposals;
    all_proposals.sort_by(|a, b| b.date.cmp(&a.date));

    let total_commits = all_commits.len();
    let repos_with_commits: Vec<&str> = config
        .projects
        .iter()
        .map(|p| p.name.as_str())
        .filter(|name| all_commits.iter().any(|c| c.project == *name))
        .collect();
    let total_repos = repos_with_commits.len();
    let total_proposals = all_proposals.len();
    let lines_added: usize = all_commits.iter().map(|c| c.insertions).sum();
    let lines_removed: usize = all_commits.iter().map(|c| c.deletions).sum();

    let unique_dates: HashSet<&str> = all_commits.iter().map(|c| &c.date[..10]).collect();
    let unique_date_count = unique_dates.len();
    let avg_daily_lines = if unique_date_count > 0 {
        (lines_added + lines_removed) as f64 / unique_date_count as f64
    } else {
        0.0
    };

    let date_range = if all_commits.is_empty() {
        "N/A".to_string()
    } else {
        let min_date = all_commits.iter().map(|c| &c.date).min().unwrap();
        let max_date = all_commits.iter().map(|c| &c.date).max().unwrap();
        format!("{} ~ {}", min_date, max_date)
    };

    let timeline = build_timeline(&all_commits);
    let type_breakdown = build_type_breakdown(&all_commits);

    let projects: Vec<ProjectData> = config
        .projects
        .iter()
        .map(|p| {
            build_project_data(
                &p.name,
                p.description.as_deref().unwrap_or(""),
                p.url.as_deref(),
                &all_commits,
                &all_proposals,
            )
        })
        .filter(|p| p.commit_count > 0 || p.proposal_count > 0)
        .collect();

    let test_metrics = collect_test_metrics(config);
    let author = author_filter.unwrap_or("ALL").to_string();

    Ok(ShowcaseData {
        title: config
            .title
            .clone()
            .unwrap_or_else(|| "貢獻總覽".to_string()),
        author,
        date_range,
        generated_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
        summary: Summary {
            total_commits,
            total_repos,
            total_proposals,
            lines_added,
            lines_removed,
            avg_daily_lines,
        },
        timeline,
        type_breakdown,
        projects,
        proposals: all_proposals,
        test_metrics,
        commits: all_commits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ProjectConfig};

    #[test]
    fn test_parse_conventional_commit_feat() {
        let (t, s) = parse_conventional_commit("feat: add login");
        assert_eq!(t, "feat");
        assert_eq!(s, "");
    }

    #[test]
    fn test_parse_conventional_commit_with_scope() {
        let (t, s) = parse_conventional_commit("fix(auth): token refresh");
        assert_eq!(t, "fix");
        assert_eq!(s, "auth");
    }

    #[test]
    fn test_parse_conventional_commit_breaking() {
        let (t, s) = parse_conventional_commit("feat!: remove API");
        assert_eq!(t, "feat");
        assert_eq!(s, "");
    }

    #[test]
    fn test_parse_conventional_commit_non_matching() {
        let (t, s) = parse_conventional_commit("Initial commit");
        assert_eq!(t, "other");
        assert_eq!(s, "");
    }

    #[test]
    fn test_parse_conventional_commit_merge() {
        let (t, s) = parse_conventional_commit("Merge branch 'dev'");
        assert_eq!(t, "other");
        assert_eq!(s, "");
    }

    #[test]
    fn test_type_label_known_types() {
        assert_eq!(type_label("feat"), "新功能");
        assert_eq!(type_label("fix"), "錯誤修復");
        assert_eq!(type_label("docs"), "文件");
        assert_eq!(type_label("refactor"), "重構");
        assert_eq!(type_label("test"), "測試");
        assert_eq!(type_label("chore"), "維護");
        assert_eq!(type_label("ci"), "CI/CD");
        assert_eq!(type_label("build"), "建構");
        assert_eq!(type_label("style"), "格式");
        assert_eq!(type_label("perf"), "效能");
    }

    #[test]
    fn test_type_label_unknown_fallback() {
        assert_eq!(type_label("unknown"), "其他");
        assert_eq!(type_label("random"), "其他");
        assert_eq!(type_label(""), "其他");
    }

    #[test]
    fn test_build_type_breakdown_no_duplicate_other() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "initial".to_string(),
                scope: "".to_string(),
                subject: "Initial commit".to_string(),
                project: "test".to_string(),
                insertions: 10,
                deletions: 5,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "merge".to_string(),
                scope: "".to_string(),
                subject: "Merge branch".to_string(),
                project: "test".to_string(),
                insertions: 20,
                deletions: 3,
            },
            CommitEntry {
                hash: "c".to_string(),
                date: "2024-01-03".to_string(),
                commit_type: "unknown".to_string(),
                scope: "".to_string(),
                subject: "Unknown change".to_string(),
                project: "test".to_string(),
                insertions: 7,
                deletions: 2,
            },
        ];
        let breakdown = build_type_breakdown(&commits);
        let other_entries: Vec<_> = breakdown.iter().filter(|b| b.label == "其他").collect();
        assert_eq!(
            other_entries.len(),
            1,
            "Should have exactly one '其他' entry"
        );
        assert_eq!(other_entries[0].lines, 10 + 5 + 20 + 3 + 7 + 2);
    }

    #[test]
    fn test_build_timeline_weekly_aggregation() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: "".to_string(),
                subject: "feat: A".to_string(),
                project: "test".to_string(),
                insertions: 10,
                deletions: 5,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: "".to_string(),
                subject: "fix: B".to_string(),
                project: "test".to_string(),
                insertions: 20,
                deletions: 3,
            },
            CommitEntry {
                hash: "c".to_string(),
                date: "2024-01-15".to_string(),
                commit_type: "docs".to_string(),
                scope: "".to_string(),
                subject: "docs: C".to_string(),
                project: "test".to_string(),
                insertions: 7,
                deletions: 2,
            },
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.len() >= 2, "Should have at least 2 week buckets");
        let total: usize = timeline.iter().map(|t| t.lines).sum();
        assert_eq!(total, 10 + 5 + 20 + 3 + 7 + 2);
    }

    #[test]
    fn test_apply_filters_date_range() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: "".to_string(),
                subject: "A".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-06-15".to_string(),
                commit_type: "fix".to_string(),
                scope: "".to_string(),
                subject: "B".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "c".to_string(),
                date: "2024-12-31".to_string(),
                commit_type: "docs".to_string(),
                scope: "".to_string(),
                subject: "C".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: Some("2024-03-01".to_string()),
                until: Some("2024-09-01".to_string()),
                types: None,
                exclude_hashes: None,
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].hash, "b");
    }

    #[test]
    fn test_apply_filters_date_range_inclusive() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: "".to_string(),
                subject: "A".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-12-31".to_string(),
                commit_type: "fix".to_string(),
                scope: "".to_string(),
                subject: "B".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: Some("2024-01-01".to_string()),
                until: Some("2024-12-31".to_string()),
                types: None,
                exclude_hashes: None,
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 2, "Boundary dates should be inclusive");
    }

    #[test]
    fn test_apply_filters_type_filter() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: "".to_string(),
                subject: "A".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: "".to_string(),
                subject: "B".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "c".to_string(),
                date: "2024-01-03".to_string(),
                commit_type: "docs".to_string(),
                scope: "".to_string(),
                subject: "C".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: None,
                until: None,
                types: Some(vec!["feat".to_string(), "fix".to_string()]),
                exclude_hashes: None,
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_shortstat_parsing_insertions_only() {
        let (ins, del) = parse_shortstat("3 files changed, 42 insertions(+)");
        assert_eq!(ins, 42);
        assert_eq!(del, 0);
    }

    #[test]
    fn test_shortstat_parsing_deletions_only() {
        let (ins, del) = parse_shortstat("1 file changed, 5 deletions(-)");
        assert_eq!(ins, 0);
        assert_eq!(del, 5);
    }

    #[test]
    fn test_shortstat_parsing_both() {
        let (ins, del) = parse_shortstat("2 files changed, 10 insertions(+), 3 deletions(-)");
        assert_eq!(ins, 10);
        assert_eq!(del, 3);
    }

    #[test]
    fn test_avg_daily_lines_multiple_dates() {
        let commits = [
            CommitEntry {
                hash: "a1".to_string(),
                date: "2025-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: a".to_string(),
                project: "p1".to_string(),
                insertions: 100,
                deletions: 50,
            },
            CommitEntry {
                hash: "a2".to_string(),
                date: "2025-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: b".to_string(),
                project: "p1".to_string(),
                insertions: 200,
                deletions: 100,
            },
            CommitEntry {
                hash: "a3".to_string(),
                date: "2025-01-03".to_string(),
                commit_type: "docs".to_string(),
                scope: String::new(),
                subject: "docs: c".to_string(),
                project: "p1".to_string(),
                insertions: 50,
                deletions: 0,
            },
        ];
        // 3 unique dates, total lines = 100+50+200+100+50+0 = 500
        let unique_dates: std::collections::HashSet<&str> =
            commits.iter().map(|c| &c.date[..10]).collect();
        let lines_added: usize = commits.iter().map(|c| c.insertions).sum();
        let lines_removed: usize = commits.iter().map(|c| c.deletions).sum();
        let avg = (lines_added + lines_removed) as f64 / unique_dates.len() as f64;
        assert!((avg - 166.666).abs() < 1.0);
        assert_eq!(unique_dates.len(), 3);
    }

    #[test]
    fn test_avg_daily_lines_same_date() {
        let commits = [
            CommitEntry {
                hash: "b1".to_string(),
                date: "2025-03-15".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: x".to_string(),
                project: "p1".to_string(),
                insertions: 40,
                deletions: 10,
            },
            CommitEntry {
                hash: "b2".to_string(),
                date: "2025-03-15".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: y".to_string(),
                project: "p1".to_string(),
                insertions: 60,
                deletions: 20,
            },
        ];
        let unique_dates: std::collections::HashSet<&str> =
            commits.iter().map(|c| &c.date[..10]).collect();
        assert_eq!(unique_dates.len(), 1);
        let lines_added: usize = commits.iter().map(|c| c.insertions).sum();
        let lines_removed: usize = commits.iter().map(|c| c.deletions).sum();
        let avg = (lines_added + lines_removed) as f64 / unique_dates.len() as f64;
        assert!((avg - 130.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_avg_daily_lines_no_commits() {
        let commits: Vec<CommitEntry> = Vec::new();
        let unique_dates: std::collections::HashSet<&str> =
            commits.iter().map(|c| &c.date[..10]).collect();
        let avg = if unique_dates.is_empty() {
            0.0
        } else {
            let lines_added: usize = commits.iter().map(|c| c.insertions).sum();
            let lines_removed: usize = commits.iter().map(|c| c.deletions).sum();
            (lines_added + lines_removed) as f64 / unique_dates.len() as f64
        };
        assert!((avg - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_stat_commits_contribute_zero_to_timeline() {
        let commits = vec![CommitEntry {
            hash: "z1".to_string(),
            date: "2024-03-01".to_string(),
            commit_type: "feat".to_string(),
            scope: String::new(),
            subject: "feat: empty".to_string(),
            project: "test".to_string(),
            insertions: 0,
            deletions: 0,
        }];
        let timeline = build_timeline(&commits);
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].lines, 0);
    }

    #[test]
    fn test_all_zero_timeline_no_division_by_zero() {
        let commits = vec![
            CommitEntry {
                hash: "z1".to_string(),
                date: "2024-03-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: empty".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "z2".to_string(),
                date: "2024-03-08".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: empty".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let timeline = build_timeline(&commits);
        for entry in &timeline {
            assert_eq!(
                entry.height, 0.0,
                "Height should be 0.0 when all lines are zero"
            );
        }
    }

    #[test]
    fn test_all_zero_type_breakdown_no_division_by_zero() {
        let commits = vec![
            CommitEntry {
                hash: "z1".to_string(),
                date: "2024-03-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: empty".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "z2".to_string(),
                date: "2024-03-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: empty".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let breakdown = build_type_breakdown(&commits);
        for entry in &breakdown {
            assert_eq!(
                entry.percentage, 0.0,
                "Percentage should be 0.0 when all lines are zero"
            );
        }
    }

    #[test]
    fn test_type_breakdown_ordering_descending_by_lines() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: big".to_string(),
                project: "test".to_string(),
                insertions: 100,
                deletions: 50,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: small".to_string(),
                project: "test".to_string(),
                insertions: 5,
                deletions: 2,
            },
            CommitEntry {
                hash: "c".to_string(),
                date: "2024-01-03".to_string(),
                commit_type: "docs".to_string(),
                scope: String::new(),
                subject: "docs: medium".to_string(),
                project: "test".to_string(),
                insertions: 30,
                deletions: 10,
            },
        ];
        let breakdown = build_type_breakdown(&commits);
        assert_eq!(breakdown.len(), 3);
        assert_eq!(breakdown[0].lines, 150); // feat: 100+50
        assert_eq!(breakdown[1].lines, 40); // docs: 30+10
        assert_eq!(breakdown[2].lines, 7); // fix: 5+2
    }

    #[test]
    fn test_project_top_types_lines_semantics() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: big fix".to_string(),
                project: "proj".to_string(),
                insertions: 200,
                deletions: 100,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: small feature".to_string(),
                project: "proj".to_string(),
                insertions: 50,
                deletions: 10,
            },
            CommitEntry {
                hash: "c".to_string(),
                date: "2024-01-03".to_string(),
                commit_type: "docs".to_string(),
                scope: String::new(),
                subject: "docs: update".to_string(),
                project: "proj".to_string(),
                insertions: 20,
                deletions: 5,
            },
        ];
        let data = build_project_data("proj", "Test project", None, &commits, &[]);
        // top_types ordered by descending lines
        assert_eq!(data.top_types.len(), 3);
        assert_eq!(data.top_types[0].commit_type, "fix");
        assert_eq!(data.top_types[0].lines, 300); // 200+100
        assert_eq!(data.top_types[1].commit_type, "feat");
        assert_eq!(data.top_types[1].lines, 60); // 50+10
        assert_eq!(data.top_types[2].commit_type, "docs");
        assert_eq!(data.top_types[2].lines, 25); // 20+5
        // percentages based on total lines (385)
        let total = 385.0_f64;
        assert!((data.top_types[0].percentage - 300.0 / total * 100.0).abs() < 0.01);
    }

    #[test]
    fn test_project_top_types_truncates_to_five() {
        let types = ["feat", "fix", "docs", "test", "chore", "ci", "style"];
        let commits: Vec<CommitEntry> = types
            .iter()
            .enumerate()
            .map(|(i, t)| CommitEntry {
                hash: format!("{}", i),
                date: "2024-01-01".to_string(),
                commit_type: t.to_string(),
                scope: String::new(),
                subject: format!("{}: x", t),
                project: "proj".to_string(),
                insertions: (i + 1) * 10,
                deletions: 0,
            })
            .collect();
        let data = build_project_data("proj", "Test project", None, &commits, &[]);
        assert_eq!(data.top_types.len(), 5);
        // First should be the largest lines (style: 70, ci: 60, chore: 50, test: 40, docs: 30)
        assert!(data.top_types[0].lines >= data.top_types[1].lines);
    }

    // --- Wiki docs override tests ---

    /// Helper that simulates the override logic in collect_git_commits_filtered.
    fn apply_wiki_override(project_name: &str, subject: &str) -> (String, String) {
        let (mut commit_type, scope) = parse_conventional_commit(subject);
        if project_name.ends_with(".wiki") {
            commit_type = "docs".to_string();
        }
        (commit_type, scope)
    }

    #[test]
    fn test_wiki_override_feat_becomes_docs() {
        let (t, s) = apply_wiki_override("my-project.wiki", "feat(nav): add sidebar");
        assert_eq!(t, "docs");
        assert_eq!(s, "nav");
    }

    #[test]
    fn test_wiki_override_non_conventional_becomes_docs() {
        let (t, s) = apply_wiki_override("my-project.wiki", "Update architecture page");
        assert_eq!(t, "docs");
        assert_eq!(s, "");
    }

    #[test]
    fn test_wiki_override_already_docs_stays_docs() {
        let (t, s) = apply_wiki_override("my-project.wiki", "docs: update readme");
        assert_eq!(t, "docs");
        assert_eq!(s, "");
    }

    #[test]
    fn test_non_wiki_project_not_overridden() {
        let (t, _) = apply_wiki_override("my-frontend", "feat: add map");
        assert_eq!(t, "feat");
    }

    #[test]
    fn test_contains_wiki_but_not_suffix_not_overridden() {
        let (t, _) = apply_wiki_override("my-project.wiki-tools", "feat: add parser");
        assert_eq!(t, "feat");
    }

    #[test]
    fn test_uppercase_wiki_not_overridden() {
        let (t, _) = apply_wiki_override("my-project.WIKI", "feat: add page");
        assert_eq!(t, "feat");
    }

    #[test]
    fn test_apply_filters_excludes_hash() {
        let commits = vec![
            CommitEntry {
                hash: "aaa111".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: keep".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "bbb222".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: drop".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: None,
                until: None,
                types: None,
                exclude_hashes: Some(vec!["bbb222".to_string()]),
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].hash, "aaa111");
    }

    #[test]
    fn test_apply_filters_retains_all_when_exclude_hashes_none() {
        let commits = vec![
            CommitEntry {
                hash: "aaa111".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: a".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "bbb222".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: b".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: None,
                until: None,
                types: None,
                exclude_hashes: None,
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_apply_filters_retains_all_when_exclude_hashes_empty_vec() {
        let commits = vec![CommitEntry {
            hash: "aaa111".to_string(),
            date: "2024-01-01".to_string(),
            commit_type: "feat".to_string(),
            scope: String::new(),
            subject: "feat: a".to_string(),
            project: "test".to_string(),
            insertions: 0,
            deletions: 0,
        }];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: None,
                until: None,
                types: None,
                exclude_hashes: Some(vec![]),
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(
            filtered.len(),
            1,
            "Empty exclude list should retain all commits"
        );
    }

    #[test]
    fn test_apply_filters_retains_all_when_no_matching_hash() {
        let commits = vec![
            CommitEntry {
                hash: "aaa111".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: a".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
            CommitEntry {
                hash: "bbb222".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: b".to_string(),
                project: "test".to_string(),
                insertions: 0,
                deletions: 0,
            },
        ];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: None,
                until: None,
                types: None,
                exclude_hashes: Some(vec!["zzz999".to_string()]),
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_apply_filters_short_prefix_does_not_exclude() {
        let commits = vec![CommitEntry {
            hash: "dd33ee63950bb49a284de835528343561f1a70d5".to_string(),
            date: "2024-01-01".to_string(),
            commit_type: "feat".to_string(),
            scope: String::new(),
            subject: "feat: a".to_string(),
            project: "test".to_string(),
            insertions: 0,
            deletions: 0,
        }];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: None,
                until: None,
                types: None,
                exclude_hashes: Some(vec!["dd33ee63".to_string()]),
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 1, "Short prefix should not match full hash");
    }

    #[test]
    fn test_detect_framework_pytest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[tool.pytest]\n[project]\ndependencies = [\"pytest\"]",
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "pytest");
    }

    #[test]
    fn test_detect_framework_none() {
        let dir = tempfile::tempdir().unwrap();
        // Empty dir, no manifest files
        assert_eq!(detect_framework(dir.path()), "none");
    }

    #[test]
    fn test_discover_test_files_pytest() {
        let dir = tempfile::tempdir().unwrap();
        let tests_dir = dir.path().join("tests");
        std::fs::create_dir(&tests_dir).unwrap();
        std::fs::write(tests_dir.join("test_auth.py"), "def test_login(): pass").unwrap();
        std::fs::write(tests_dir.join("test_api.py"), "def test_get(): pass").unwrap();
        std::fs::write(tests_dir.join("utils.py"), "# not a test").unwrap();
        let files = discover_test_files(dir.path(), "pytest");
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_count_test_cases_python() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test_example.py");
        std::fs::write(
            &test_file,
            "def test_a():\n    pass\n\nasync def test_b():\n    pass\n\ndef helper():\n    pass\n",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "pytest"), 2);
    }

    #[test]
    fn test_count_test_cases_js() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("app.test.ts");
        std::fs::write(
            &test_file,
            "describe('app', () => {\n  it('should work', () => {});\n  test('another', () => {});\n});",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "vitest"), 2);
    }

    #[test]
    fn test_discover_coverage_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(discover_coverage(dir.path(), None), None);
    }

    #[test]
    fn test_parse_cobertura_xml() {
        let dir = tempfile::tempdir().unwrap();
        let xml_path = dir.path().join("coverage.xml");
        std::fs::write(
            &xml_path,
            r#"<?xml version="1.0" ?><coverage line-rate="0.85" branch-rate="0.70"></coverage>"#,
        )
        .unwrap();
        assert_eq!(parse_cobertura_xml(&xml_path), Some(85.0));
    }

    #[test]
    fn test_parse_istanbul_json() {
        let dir = tempfile::tempdir().unwrap();
        let json_dir = dir.path().join("coverage");
        std::fs::create_dir(&json_dir).unwrap();
        let json_path = json_dir.join("coverage-summary.json");
        std::fs::write(
            &json_path,
            r#"{"total":{"lines":{"pct":72.5},"statements":{"pct":70.0}}}"#,
        )
        .unwrap();
        assert_eq!(parse_istanbul_json(&json_path), Some(72.5));
    }

    #[test]
    fn test_discover_coverage_with_explicit_xml_path() {
        let dir = tempfile::tempdir().unwrap();
        let xml_path = dir.path().join("coverage.xml");
        std::fs::write(
            &xml_path,
            r#"<?xml version="1.0" ?><coverage line-rate="0.92"></coverage>"#,
        )
        .unwrap();
        assert_eq!(
            discover_coverage(dir.path(), Some("coverage.xml")),
            Some(92.0)
        );
    }

    #[test]
    fn test_discover_coverage_with_explicit_json_path() {
        let dir = tempfile::tempdir().unwrap();
        let cov_dir = dir.path().join("coverage");
        std::fs::create_dir(&cov_dir).unwrap();
        std::fs::write(
            cov_dir.join("coverage-summary.json"),
            r#"{"total":{"lines":{"pct":66.3}}}"#,
        )
        .unwrap();
        assert_eq!(
            discover_coverage(dir.path(), Some("coverage/coverage-summary.json")),
            Some(66.3)
        );
    }

    #[test]
    fn test_discover_coverage_auto_fallback() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("coverage.xml"),
            r#"<coverage line-rate="0.78"></coverage>"#,
        )
        .unwrap();
        assert_eq!(discover_coverage(dir.path(), None), Some(78.0));
    }

    // ========================================================================
    // collect_proposals() tests
    // ========================================================================

    #[test]
    fn test_collect_proposals_empty_archive() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");
        std::fs::create_dir_all(&archive).unwrap();
        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_proposals_no_archive_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_proposals_with_valid_entries() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");

        // Create two archive entries
        let entry1 = archive.join("2024-06-15-add-auth");
        std::fs::create_dir_all(&entry1).unwrap();
        std::fs::write(
            entry1.join(".openspec.yaml"),
            "description: \"Add authentication module\"",
        )
        .unwrap();
        std::fs::write(
            entry1.join("tasks.md"),
            "- [x] task 1\n- [x] task 2\n- [ ] task 3\n",
        )
        .unwrap();

        let entry2 = archive.join("2024-07-20-fix-bug");
        std::fs::create_dir_all(&entry2).unwrap();
        std::fs::write(
            entry2.join(".openspec.yaml"),
            "description: 'Fix critical bug'",
        )
        .unwrap();
        std::fs::write(entry2.join("tasks.md"), "- [x] fix it\n").unwrap();

        let result = collect_proposals(dir.path().to_str().unwrap(), "my-proj");
        assert_eq!(result.len(), 2);
        // Sorted most recent first
        assert_eq!(result[0].date, "2024-07-20");
        assert_eq!(result[0].slug, "fix-bug");
        assert_eq!(result[0].project, "my-proj");
        assert_eq!(result[0].description, "Fix critical bug");
        assert_eq!(result[0].task_count, 1);

        assert_eq!(result[1].date, "2024-06-15");
        assert_eq!(result[1].slug, "add-auth");
        assert_eq!(result[1].task_count, 2); // only [x] tasks
        assert_eq!(result[1].description, "Add authentication module");
    }

    #[test]
    fn test_collect_proposals_non_date_prefixed_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");
        let bad_entry = archive.join("not-a-date-slug");
        std::fs::create_dir_all(&bad_entry).unwrap();
        std::fs::write(bad_entry.join(".openspec.yaml"), "description: nope").unwrap();
        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_proposals_missing_openspec_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");
        let entry = archive.join("2024-01-01-no-yaml");
        std::fs::create_dir_all(&entry).unwrap();
        std::fs::write(entry.join("tasks.md"), "- [x] done\n").unwrap();

        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "");
        assert_eq!(result[0].task_count, 1);
    }

    #[test]
    fn test_collect_proposals_missing_tasks_md() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");
        let entry = archive.join("2024-05-10-no-tasks");
        std::fs::create_dir_all(&entry).unwrap();
        std::fs::write(entry.join(".openspec.yaml"), "description: hello").unwrap();

        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].task_count, 0);
        assert_eq!(result[0].description, "hello");
    }

    #[test]
    fn test_collect_proposals_non_directory_entries_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");
        std::fs::create_dir_all(&archive).unwrap();
        // Create a file (not a directory)
        std::fs::write(archive.join("2024-01-01-file-not-dir"), "content").unwrap();
        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_proposals_sorting_most_recent_first() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("openspec/changes/archive");
        for date in &["2024-01-01", "2024-12-31", "2024-06-15"] {
            let entry = archive.join(format!("{}-slug", date));
            std::fs::create_dir_all(&entry).unwrap();
        }
        let result = collect_proposals(dir.path().to_str().unwrap(), "proj");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].date, "2024-12-31");
        assert_eq!(result[1].date, "2024-06-15");
        assert_eq!(result[2].date, "2024-01-01");
    }

    // ========================================================================
    // collect_git_commits_filtered() tests
    // ========================================================================

    fn init_temp_git_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(repo)
                .env("GIT_AUTHOR_NAME", "Test User")
                .env("GIT_AUTHOR_EMAIL", "test@example.com")
                .env("GIT_COMMITTER_NAME", "Test User")
                .env("GIT_COMMITTER_EMAIL", "test@example.com")
                .output()
                .unwrap()
        };
        run(&["init", "--initial-branch", "main"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test User"]);
        dir
    }

    fn git_commit(repo: &Path, filename: &str, content: &str, message: &str) {
        std::fs::write(repo.join(filename), content).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo)
            .env("GIT_AUTHOR_NAME", "Test User")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test User")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo)
            .env("GIT_AUTHOR_NAME", "Test User")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test User")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .unwrap();
    }

    #[test]
    fn test_collect_git_commits_basic() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "file.txt", "hello", "feat: initial commit");

        let commits =
            collect_git_commits_filtered(repo.to_str().unwrap(), "test-proj", None, Some("main"))
                .unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].commit_type, "feat");
        assert_eq!(commits[0].subject, "feat: initial commit");
        assert_eq!(commits[0].project, "test-proj");
        assert!(!commits[0].hash.is_empty());
        assert_eq!(commits[0].date.len(), 10); // YYYY-MM-DD
    }

    #[test]
    fn test_collect_git_commits_parses_shortstat() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "a.txt", "line1\nline2\nline3\n", "feat: add file");

        let commits =
            collect_git_commits_filtered(repo.to_str().unwrap(), "proj", None, Some("main"))
                .unwrap();
        assert_eq!(commits.len(), 1);
        assert!(commits[0].insertions > 0, "Should have insertions");
    }

    #[test]
    fn test_collect_git_commits_multiple() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "a.txt", "hello", "feat: add a");
        git_commit(repo, "b.txt", "world", "fix: add b");
        git_commit(repo, "c.txt", "test", "docs: add c");

        let commits =
            collect_git_commits_filtered(repo.to_str().unwrap(), "proj", None, Some("main"))
                .unwrap();
        assert_eq!(commits.len(), 3);
    }

    #[test]
    fn test_collect_git_commits_wiki_override() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "page.md", "# Hello", "feat: add page");

        let commits = collect_git_commits_filtered(
            repo.to_str().unwrap(),
            "project.wiki",
            None,
            Some("main"),
        )
        .unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].commit_type, "docs");
    }

    #[test]
    fn test_collect_git_commits_nonexistent_path() {
        let result = collect_git_commits_filtered(
            "/nonexistent/path/that/does/not/exist",
            "proj",
            None,
            None,
        )
        .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_git_commits_path_without_git() {
        let dir = tempfile::tempdir().unwrap();
        // No .git directory
        let result =
            collect_git_commits_filtered(dir.path().to_str().unwrap(), "proj", None, None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_git_commits_author_filter() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "a.txt", "hello", "feat: add a");

        // Filter by matching author
        let commits = collect_git_commits_filtered(
            repo.to_str().unwrap(),
            "proj",
            Some("Test User"),
            Some("main"),
        )
        .unwrap();
        assert_eq!(commits.len(), 1);

        // Filter by non-matching author
        let commits = collect_git_commits_filtered(
            repo.to_str().unwrap(),
            "proj",
            Some("Nobody"),
            Some("main"),
        )
        .unwrap();
        assert!(commits.is_empty());
    }

    #[test]
    fn test_collect_git_commits_all_branches_when_no_branch() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "a.txt", "hello", "feat: on main");

        // No branch specified → --all
        let commits =
            collect_git_commits_filtered(repo.to_str().unwrap(), "proj", None, None).unwrap();
        assert!(!commits.is_empty());
    }

    #[test]
    fn test_collect_git_commits_branch_filter() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "a.txt", "hello", "feat: on main");

        // Create a new branch with an extra commit
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(repo)
            .output()
            .unwrap();
        git_commit(repo, "b.txt", "world", "fix: on feature");

        // main should have 1 commit
        let main_commits =
            collect_git_commits_filtered(repo.to_str().unwrap(), "proj", None, Some("main"))
                .unwrap();
        assert_eq!(main_commits.len(), 1);

        // feature should have 2 commits
        let feat_commits =
            collect_git_commits_filtered(repo.to_str().unwrap(), "proj", None, Some("feature"))
                .unwrap();
        assert_eq!(feat_commits.len(), 2);
    }

    #[test]
    fn test_collect_git_commits_invalid_branch() {
        let dir = init_temp_git_repo();
        let repo = dir.path();
        git_commit(repo, "a.txt", "hello", "feat: initial");

        let commits = collect_git_commits_filtered(
            repo.to_str().unwrap(),
            "proj",
            None,
            Some("nonexistent-branch"),
        )
        .unwrap();
        assert!(commits.is_empty());
    }

    // ========================================================================
    // detect_framework() tests
    // ========================================================================

    #[test]
    fn test_detect_framework_cargo() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        assert_eq!(detect_framework(dir.path()), "cargo test");
    }

    #[test]
    fn test_detect_framework_vitest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies":{"vitest":"^1.0"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "vitest");
    }

    #[test]
    fn test_detect_framework_jest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies":{"jest":"^29"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "jest");
    }

    #[test]
    fn test_detect_framework_pyproject_without_pytest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\nname = \"x\"\ndependencies = [\"requests\"]",
        )
        .unwrap();
        // pyproject.toml without pytest — falls through. No Cargo.toml, no package.json.
        assert_eq!(detect_framework(dir.path()), "none");
    }

    #[test]
    fn test_detect_framework_priority_pytest_over_cargo() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[tool.pytest]\n[project]\ndependencies = [\"pytest\"]",
        )
        .unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        assert_eq!(detect_framework(dir.path()), "pytest");
    }

    #[test]
    fn test_detect_framework_priority_cargo_over_vitest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies":{"vitest":"^1.0"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "cargo test");
    }

    #[test]
    fn test_detect_framework_priority_vitest_over_jest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies":{"vitest":"^1.0","jest":"^29"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "vitest");
    }

    #[test]
    fn test_detect_framework_deno_json_with_vitest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("deno.json"),
            r#"{"imports":{"vitest":"npm:vitest@^4.0.0"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "vitest");
    }

    #[test]
    fn test_detect_framework_deno_jsonc_with_vitest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("deno.jsonc"),
            r#"{"imports":{"vitest":"npm:vitest@^4.0.0"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "vitest");
    }

    #[test]
    fn test_detect_framework_deno_json_without_vitest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("deno.json"),
            r#"{"tasks":{"test":"deno test tests/"}}"#,
        )
        .unwrap();
        assert_eq!(detect_framework(dir.path()), "deno test");
    }

    #[test]
    fn test_detect_framework_priority_cargo_over_deno() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        std::fs::write(dir.path().join("deno.json"), "{}").unwrap();
        assert_eq!(detect_framework(dir.path()), "cargo test");
    }

    // ========================================================================
    // is_excluded_dir() tests
    // ========================================================================

    #[test]
    fn test_is_excluded_dir_all_excluded() {
        let excluded = [
            "node_modules",
            "target",
            ".venv",
            "__pycache__",
            ".git",
            "dist",
            ".tox",
        ];
        for name in &excluded {
            assert!(is_excluded_dir(name), "'{}' should be excluded", name);
        }
    }

    #[test]
    fn test_is_excluded_dir_non_excluded() {
        let allowed = ["src", "tests", "lib", "docs", "build", "bin"];
        for name in &allowed {
            assert!(!is_excluded_dir(name), "'{}' should not be excluded", name);
        }
    }

    // ========================================================================
    // is_test_file() tests
    // ========================================================================

    #[test]
    fn test_is_test_file_pytest_patterns() {
        assert!(is_test_file(Path::new("test_foo.py"), "pytest"));
        assert!(is_test_file(Path::new("foo_test.py"), "pytest"));
        assert!(!is_test_file(Path::new("foo.py"), "pytest"));
        assert!(!is_test_file(Path::new("test_foo.rs"), "pytest"));
    }

    #[test]
    fn test_is_test_file_vitest_jest_patterns() {
        assert!(is_test_file(Path::new("foo.test.ts"), "vitest"));
        assert!(is_test_file(Path::new("bar.spec.js"), "jest"));
        assert!(is_test_file(Path::new("baz.test.tsx"), "vitest"));
        assert!(!is_test_file(Path::new("foo.ts"), "vitest"));
        assert!(!is_test_file(Path::new("foo.ts"), "jest"));
    }

    #[test]
    fn test_is_test_file_cargo_patterns() {
        assert!(is_test_file(Path::new("foo.rs"), "cargo test"));
        assert!(!is_test_file(Path::new("foo.py"), "cargo test"));
        // No extension → false
        assert!(!is_test_file(Path::new("Makefile"), "cargo test"));
    }

    #[test]
    fn test_is_test_file_unknown_framework() {
        assert!(!is_test_file(Path::new("test_foo.py"), "unknown"));
        assert!(!is_test_file(Path::new("foo.rs"), "none"));
    }

    #[test]
    fn test_is_test_file_deno_test_patterns() {
        assert!(is_test_file(Path::new("parser_test.ts"), "deno test"));
        assert!(is_test_file(Path::new("parser_test.js"), "deno test"));
        assert!(is_test_file(Path::new("parser_test.mjs"), "deno test"));
        assert!(is_test_file(Path::new("foo.test.ts"), "deno test"));
        assert!(!is_test_file(Path::new("parser.ts"), "deno test"));
        assert!(!is_test_file(Path::new("parser.js"), "deno test"));
    }

    // ========================================================================
    // discover_test_files() tests
    // ========================================================================

    #[test]
    fn test_discover_test_files_cargo() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        // File with #[test] → should be discovered
        std::fs::write(
            src.join("lib.rs"),
            "fn foo() {}\n#[cfg(test)]\nmod tests {\n#[test]\nfn it_works() {}\n}\n",
        )
        .unwrap();
        // File without tests → should NOT be discovered
        std::fs::write(src.join("utils.rs"), "pub fn helper() -> i32 { 42 }\n").unwrap();
        let files = discover_test_files(dir.path(), "cargo test");
        assert_eq!(files.len(), 1);
        assert!(files[0].to_str().unwrap().contains("lib.rs"));
    }

    #[test]
    fn test_discover_test_files_skips_excluded_dirs() {
        let dir = tempfile::tempdir().unwrap();
        // Put a .rs file with #[test] in target/ — should be excluded
        let target_dir = dir.path().join("target/debug");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("test.rs"), "#[test]\nfn t() {}\n").unwrap();
        // And one in src/ — should be found
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("main.rs"), "#[test]\nfn t() {}\n").unwrap();

        let files = discover_test_files(dir.path(), "cargo test");
        assert_eq!(files.len(), 1);
        assert!(
            files[0]
                .strip_prefix(dir.path())
                .unwrap()
                .ends_with(Path::new("src").join("main.rs"))
        );
    }

    #[test]
    fn test_discover_test_files_vitest() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("app.test.ts"), "test('works', () => {})").unwrap();
        std::fs::write(src.join("app.ts"), "export const foo = 1;").unwrap();
        // node_modules should be excluded
        let nm = dir.path().join("node_modules/pkg");
        std::fs::create_dir_all(&nm).unwrap();
        std::fs::write(nm.join("index.test.js"), "test('x', () => {})").unwrap();

        let files = discover_test_files(dir.path(), "vitest");
        assert_eq!(files.len(), 1);
    }

    // ========================================================================
    // count_test_cases() tests
    // ========================================================================

    #[test]
    fn test_count_test_cases_cargo() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("tests.rs");
        std::fs::write(
            &test_file,
            "#[test]\nfn a() {}\n#[test]\nfn b() {}\n#[tokio::test]\nasync fn c() {}\n#[rstest]\nfn d() {}\n",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "cargo test"), 4);
    }

    #[test]
    fn test_count_test_cases_vitest_each() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("app.test.ts");
        std::fs::write(
            &test_file,
            "it.each([1,2])('works %d', (n) => {});\ntest.each([3])('t', () => {});\n",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "vitest"), 2);
    }

    #[test]
    fn test_count_test_cases_vitest_excludes_comments() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("a.test.ts");
        std::fs::write(
            &test_file,
            "// it('commented out', () => {});\n* test('also excluded', () => {});\nit('real', () => {});\n",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "vitest"), 1);
    }

    #[test]
    fn test_count_test_cases_pytest_async() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test_async.py");
        std::fs::write(
            &test_file,
            "async def test_a():\n    pass\ndef test_b():\n    pass\ndef helper():\n    pass\n",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "pytest"), 2);
    }

    #[test]
    fn test_count_test_cases_unknown_framework() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test_file.py");
        std::fs::write(&test_file, "def test_a():\n    pass\n").unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "unknown"), 0);
    }

    #[test]
    fn test_count_test_cases_deno_test() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("parser_test.js");
        std::fs::write(
            &test_file,
            "Deno.test(\"parses basic\", () => {});\nDeno.test.only(\"only this\", () => {});\nDeno.test.ignore(\"skip\", () => {});\n// Deno.test(\"commented\", () => {});\n",
        )
        .unwrap();
        let files = vec![test_file];
        assert_eq!(count_test_cases(&files, "deno test"), 3);
    }

    // ========================================================================
    // discover_coverage() edge cases
    // ========================================================================

    #[test]
    fn test_discover_coverage_explicit_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(discover_coverage(dir.path(), Some("nonexistent.xml")), None);
    }

    #[test]
    fn test_discover_coverage_explicit_unsupported_extension() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("report.txt"), "some text").unwrap();
        assert_eq!(discover_coverage(dir.path(), Some("report.txt")), None);
    }

    #[test]
    fn test_discover_coverage_auto_fallback_to_istanbul() {
        let dir = tempfile::tempdir().unwrap();
        // No coverage.xml, only Istanbul JSON
        let cov_dir = dir.path().join("coverage");
        std::fs::create_dir(&cov_dir).unwrap();
        std::fs::write(
            cov_dir.join("coverage-summary.json"),
            r#"{"total":{"lines":{"pct":55.5}}}"#,
        )
        .unwrap();
        assert_eq!(discover_coverage(dir.path(), None), Some(55.5));
    }

    #[test]
    fn test_discover_coverage_auto_neither_found() {
        let dir = tempfile::tempdir().unwrap();
        // No coverage files at all
        assert_eq!(discover_coverage(dir.path(), None), None);
    }

    // ========================================================================
    // parse_conventional_commit() edge cases
    // ========================================================================

    #[test]
    fn test_parse_conventional_commit_empty_scope() {
        let (t, s) = parse_conventional_commit("docs(): update");
        assert_eq!(t, "docs");
        assert_eq!(s, "");
    }

    #[test]
    fn test_parse_conventional_commit_breaking_with_scope() {
        let (t, s) = parse_conventional_commit("refactor(core)!: restructure");
        assert_eq!(t, "refactor");
        assert_eq!(s, "core");
    }

    #[test]
    fn test_parse_conventional_commit_empty_string() {
        let (t, s) = parse_conventional_commit("");
        assert_eq!(t, "other");
        assert_eq!(s, "");
    }

    // ========================================================================
    // build_timeline() edge cases
    // ========================================================================

    #[test]
    fn test_build_timeline_empty_commits() {
        let timeline = build_timeline(&[]);
        assert!(timeline.is_empty());
    }

    fn make_commit(date: &str, insertions: usize, deletions: usize) -> CommitEntry {
        CommitEntry {
            hash: "abc123".to_string(),
            date: date.to_string(),
            commit_type: "feat".to_string(),
            scope: String::new(),
            subject: "test".to_string(),
            project: "test".to_string(),
            insertions,
            deletions,
        }
    }

    #[test]
    fn test_build_timeline_daily_granularity() {
        let commits = vec![
            make_commit("2025-03-10", 10, 5),
            make_commit("2025-03-12", 20, 3),
            make_commit("2025-03-15", 7, 2),
        ];
        let timeline = build_timeline(&commits);
        // Contiguous daily buckets from 03-10 to 03-15 = 6 days
        assert_eq!(timeline.len(), 6);
        assert!(timeline.iter().all(|t| t.label.len() == 10)); // %Y-%m-%d
        assert_eq!(timeline[0].label, "2025-03-10");
        assert_eq!(timeline[0].lines, 15); // 10+5
        assert_eq!(timeline[1].label, "2025-03-11");
        assert_eq!(timeline[1].lines, 0); // gap day
        assert_eq!(timeline[2].label, "2025-03-12");
        assert_eq!(timeline[2].lines, 23); // 20+3
        assert_eq!(timeline[5].label, "2025-03-15");
        assert_eq!(timeline[5].lines, 9); // 7+2
    }

    #[test]
    fn test_build_timeline_weekly_granularity() {
        let commits = vec![
            make_commit("2025-03-01", 10, 5),
            make_commit("2025-03-15", 20, 3),
            make_commit("2025-03-31", 7, 2),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
        let total: usize = timeline.iter().map(|t| t.lines).sum();
        assert_eq!(total, 10 + 5 + 20 + 3 + 7 + 2);
    }

    #[test]
    fn test_build_timeline_monthly_granularity() {
        // Span ~8 months: > 14 distinct weeks, ≤ 14 months → monthly
        let commits = vec![
            make_commit("2025-01-15", 10, 5),
            make_commit("2025-06-01", 20, 3),
            make_commit("2025-08-15", 7, 2),
        ];
        let timeline = build_timeline(&commits);
        // Contiguous monthly buckets from 2025-01 to 2025-08 = 8 months
        assert_eq!(timeline.len(), 8);
        assert!(timeline.iter().all(|t| t.label.len() == 7)); // %Y-%m
        assert_eq!(timeline[0].label, "2025-01");
        assert_eq!(timeline[0].lines, 15);
        assert_eq!(timeline[1].label, "2025-02");
        assert_eq!(timeline[1].lines, 0); // gap month
        assert_eq!(timeline[4].label, "2025-05");
        assert_eq!(timeline[4].lines, 0); // gap month
        assert_eq!(timeline[5].label, "2025-06");
        assert_eq!(timeline[5].lines, 23);
        assert_eq!(timeline[7].label, "2025-08");
        assert_eq!(timeline[7].lines, 9);
    }

    #[test]
    fn test_build_timeline_boundary_14_days() {
        // 15 inclusive days (> 14) → escalates to weekly
        let commits = vec![
            make_commit("2025-03-01", 10, 0),
            make_commit("2025-03-15", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
    }

    #[test]
    fn test_build_timeline_boundary_60_days() {
        // 61 inclusive days → > 14 days; ~9 distinct weeks ≤ 14 → weekly
        let commits = vec![
            make_commit("2025-01-01", 10, 0),
            make_commit("2025-03-02", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
    }

    #[test]
    fn test_build_timeline_boundary_61_days() {
        // 62 inclusive days → > 14 days; ~9 distinct weeks ≤ 14 → weekly
        let commits = vec![
            make_commit("2025-01-01", 10, 0),
            make_commit("2025-03-03", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
    }

    #[test]
    fn test_build_timeline_single_day() {
        let commits = vec![
            make_commit("2025-06-01", 10, 5),
            make_commit("2025-06-01", 20, 3),
        ];
        let timeline = build_timeline(&commits);
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].label, "2025-06-01");
        assert_eq!(timeline[0].lines, 10 + 5 + 20 + 3);
        assert_eq!(timeline[0].height, 100.0);
    }

    #[test]
    fn test_build_timeline_boundary_13_days() {
        // 13 days span → daily
        let commits = vec![
            make_commit("2025-03-01", 10, 0),
            make_commit("2025-03-14", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // Contiguous daily buckets: 03-01 through 03-14 = 14 days
        assert_eq!(timeline.len(), 14);
        assert_eq!(timeline[0].label, "2025-03-01");
        assert_eq!(timeline[0].lines, 10);
        assert_eq!(timeline[13].label, "2025-03-14");
        assert_eq!(timeline[13].lines, 5);
        // Gap days should have 0 lines
        assert_eq!(timeline[1].lines, 0);
    }

    #[test]
    fn test_build_timeline_quarterly_granularity() {
        // Span ~16 months: > 14 distinct weeks, > 14 months, ≤ 14 quarters → quarterly
        let commits = vec![
            make_commit("2024-01-15", 10, 5),
            make_commit("2025-04-20", 20, 3),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-Q")));
        // 2024-Q1 to 2025-Q2 = 6 quarters
        assert_eq!(timeline.len(), 6);
        assert_eq!(timeline[0].label, "2024-Q1");
        assert_eq!(timeline[0].lines, 15);
        assert_eq!(timeline[1].label, "2024-Q2");
        assert_eq!(timeline[1].lines, 0);
        assert_eq!(timeline[5].label, "2025-Q2");
        assert_eq!(timeline[5].lines, 23);
    }

    #[test]
    fn test_build_timeline_quarterly_contiguous_with_gaps() {
        // Span > 14 months, ≤ 14 quarters → quarterly with gap filling
        let commits = vec![
            make_commit("2023-01-15", 10, 0),
            make_commit("2024-06-15", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // 2023-Q1 to 2024-Q2 = 6 quarters
        assert_eq!(timeline.len(), 6);
        assert!(timeline.iter().all(|t| t.label.contains("-Q")));
        assert_eq!(timeline[0].label, "2023-Q1");
        assert_eq!(timeline[0].lines, 10);
        assert_eq!(timeline[1].label, "2023-Q2");
        assert_eq!(timeline[1].lines, 0); // gap
        assert_eq!(timeline[5].label, "2024-Q2");
        assert_eq!(timeline[5].lines, 5);
    }

    #[test]
    fn test_build_timeline_yearly_granularity() {
        // Span > 14 quarters → yearly
        let commits = vec![
            make_commit("2020-01-01", 100, 50),
            make_commit("2025-12-31", 20, 10),
        ];
        let timeline = build_timeline(&commits);
        assert_eq!(timeline.len(), 6); // 2020..2025
        assert!(timeline.iter().all(|t| t.label.len() == 4)); // YYYY
        assert_eq!(timeline[0].label, "2020");
        assert_eq!(timeline[0].lines, 150);
        assert_eq!(timeline[1].label, "2021");
        assert_eq!(timeline[1].lines, 0); // gap
        assert_eq!(timeline[5].label, "2025");
        assert_eq!(timeline[5].lines, 30);
    }

    #[test]
    fn test_build_timeline_yearly_contiguous_with_gaps() {
        // Multi-decade span
        let commits = vec![
            make_commit("2015-06-01", 10, 0),
            make_commit("2025-06-01", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert_eq!(timeline.len(), 11); // 2015..2025
        assert!(timeline.iter().all(|t| t.label.len() == 4));
        assert_eq!(timeline[0].label, "2015");
        assert_eq!(timeline[10].label, "2025");
        // All gap years should be 0
        for entry in timeline.iter().take(10).skip(1) {
            assert_eq!(entry.lines, 0);
        }
    }

    #[test]
    fn test_build_timeline_boundary_14_days_inclusive_daily() {
        // Exactly 14 inclusive days → daily
        let commits = vec![
            make_commit("2025-03-01", 10, 0),
            make_commit("2025-03-14", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert_eq!(timeline.len(), 14);
        assert!(timeline.iter().all(|t| t.label.len() == 10)); // YYYY-MM-DD
    }

    #[test]
    fn test_build_timeline_boundary_15_days_to_weekly() {
        // 15 inclusive days → > 14, escalate to weekly
        let commits = vec![
            make_commit("2025-03-01", 10, 0),
            make_commit("2025-03-15", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
    }

    #[test]
    fn test_build_timeline_boundary_14_weeks_to_weekly() {
        // Span producing exactly 14 distinct ISO weeks → weekly
        // 2025-01-06 (W02) to 2025-04-13 (W15) = 14 distinct weeks
        let commits = vec![
            make_commit("2025-01-06", 10, 0), // W02
            make_commit("2025-04-13", 5, 0),  // W15
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
        assert_eq!(timeline.len(), 14);
    }

    #[test]
    fn test_build_timeline_boundary_15_weeks_to_monthly() {
        // Span producing > 14 distinct ISO weeks → monthly (if ≤ 14 months)
        let commits = vec![
            make_commit("2025-01-01", 10, 0),
            make_commit("2025-04-30", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // Jan-Apr = 4 months, but > 14 weeks → monthly
        // Check label format: YYYY-MM (7 chars)
        assert!(timeline.iter().all(|t| t.label.len() == 7));
    }

    #[test]
    fn test_build_timeline_boundary_15_months_to_quarterly() {
        // 15 distinct months → quarterly
        let commits = vec![
            make_commit("2024-01-01", 10, 0),
            make_commit("2025-03-31", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // 15 months > 14 → quarterly. Q1 2024 to Q1 2025 = 5 quarters
        assert!(timeline.iter().all(|t| t.label.contains("-Q")));
    }

    #[test]
    fn test_build_timeline_boundary_14_months_stays_monthly() {
        // Exactly 14 distinct months → monthly
        let commits = vec![
            make_commit("2024-01-01", 10, 0),
            make_commit("2025-02-28", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // 14 months ≤ 14 → monthly
        assert_eq!(timeline.len(), 14);
        assert!(timeline.iter().all(|t| t.label.len() == 7));
    }

    #[test]
    fn test_build_timeline_boundary_14_quarters_stays_quarterly() {
        // Exactly 14 distinct quarters → quarterly
        // Q1 2022 to Q2 2025 = 14 quarters
        let commits = vec![
            make_commit("2022-01-15", 10, 0),
            make_commit("2025-06-15", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert_eq!(timeline.len(), 14);
        assert!(timeline.iter().all(|t| t.label.contains("-Q")));
    }

    #[test]
    fn test_build_timeline_boundary_15_quarters_to_yearly() {
        // 15 distinct quarters → yearly
        let commits = vec![
            make_commit("2021-01-01", 10, 0),
            make_commit("2024-09-30", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // Q1 2021 to Q3 2024 = 15 quarters > 14 → yearly
        assert!(timeline.iter().all(|t| t.label.len() == 4)); // YYYY
        assert_eq!(timeline.len(), 4); // 2021, 2022, 2023, 2024
    }

    #[test]
    fn test_build_timeline_iso_week_year_boundary() {
        // Dec 29, 2024 is ISO week 2025-W01 (Mon Dec 30 is in W01)
        // This tests that ISO week-year (%G) is used, not calendar year (%Y)
        let commits = vec![
            make_commit("2024-12-25", 10, 0),
            make_commit("2025-01-05", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        // 12 inclusive days → daily
        assert_eq!(timeline.len(), 12);
        assert!(timeline.iter().all(|t| t.label.len() == 10));
    }

    #[test]
    fn test_build_timeline_iso_week_year_boundary_weekly() {
        // Test ISO week-year boundary in weekly mode
        // Dec 20, 2024 to Jan 20, 2025 = 32 inclusive days → > 14 → weekly
        let commits = vec![
            make_commit("2024-12-20", 10, 0),
            make_commit("2025-01-20", 5, 0),
        ];
        let timeline = build_timeline(&commits);
        assert!(timeline.iter().all(|t| t.label.contains("-W")));
        // Should have labels crossing ISO year boundary:
        // 2024-W51, 2024-W52, 2025-W01, 2025-W02, 2025-W03, 2025-W04
        let has_2024 = timeline.iter().any(|t| t.label.starts_with("2024-W"));
        let has_2025 = timeline.iter().any(|t| t.label.starts_with("2025-W"));
        assert!(has_2024, "Should have 2024 ISO week labels");
        assert!(has_2025, "Should have 2025 ISO week labels");
    }

    // ========================================================================
    // build_type_breakdown() edge cases
    // ========================================================================

    #[test]
    fn test_build_type_breakdown_empty_commits() {
        let breakdown = build_type_breakdown(&[]);
        assert!(breakdown.is_empty());
    }

    #[test]
    fn test_build_type_breakdown_single_type() {
        let commits = vec![CommitEntry {
            hash: "a".to_string(),
            date: "2024-01-01".to_string(),
            commit_type: "feat".to_string(),
            scope: String::new(),
            subject: "feat: thing".to_string(),
            project: "p".to_string(),
            insertions: 10,
            deletions: 5,
        }];
        let breakdown = build_type_breakdown(&commits);
        assert_eq!(breakdown.len(), 1);
        assert_eq!(breakdown[0].commit_type, "feat");
        assert!((breakdown[0].percentage - 100.0).abs() < f64::EPSILON);
    }

    // ========================================================================
    // parse_shortstat() edge cases
    // ========================================================================

    #[test]
    fn test_parse_shortstat_empty_string() {
        let (ins, del) = parse_shortstat("");
        assert_eq!(ins, 0);
        assert_eq!(del, 0);
    }

    #[test]
    fn test_parse_shortstat_whitespace_only() {
        let (ins, del) = parse_shortstat("   ");
        assert_eq!(ins, 0);
        assert_eq!(del, 0);
    }

    // ========================================================================
    // apply_filters() — proposal date filtering
    // ========================================================================

    #[test]
    fn test_apply_filters_proposals_date_range() {
        let commits = vec![];
        let proposals = vec![
            ProposalEntry {
                slug: "early".to_string(),
                date: "2024-01-01".to_string(),
                project: "proj".to_string(),
                description: "early".to_string(),
                task_count: 1,
            },
            ProposalEntry {
                slug: "mid".to_string(),
                date: "2024-06-15".to_string(),
                project: "proj".to_string(),
                description: "mid".to_string(),
                task_count: 2,
            },
            ProposalEntry {
                slug: "late".to_string(),
                date: "2024-12-31".to_string(),
                project: "proj".to_string(),
                description: "late".to_string(),
                task_count: 3,
            },
        ];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: Some(crate::config::FilterConfig {
                author: None,
                since: Some("2024-03-01".to_string()),
                until: Some("2024-09-01".to_string()),
                types: None,
                exclude_hashes: None,
            }),
        };
        let (_, filtered_proposals) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered_proposals.len(), 1);
        assert_eq!(filtered_proposals[0].slug, "mid");
    }

    #[test]
    fn test_apply_filters_no_filters() {
        let commits = vec![CommitEntry {
            hash: "a".to_string(),
            date: "2024-01-01".to_string(),
            commit_type: "feat".to_string(),
            scope: String::new(),
            subject: "feat: a".to_string(),
            project: "test".to_string(),
            insertions: 0,
            deletions: 0,
        }];
        let proposals = vec![];
        let config = Config {
            title: None,
            output: None,
            projects: vec![],
            filters: None,
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 1);
    }

    // ========================================================================
    // build_project_data() tests
    // ========================================================================

    #[test]
    fn test_build_project_data_filters_by_project() {
        let commits = vec![
            CommitEntry {
                hash: "a".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: a".to_string(),
                project: "proj-a".to_string(),
                insertions: 10,
                deletions: 5,
            },
            CommitEntry {
                hash: "b".to_string(),
                date: "2024-01-02".to_string(),
                commit_type: "fix".to_string(),
                scope: String::new(),
                subject: "fix: b".to_string(),
                project: "proj-b".to_string(),
                insertions: 20,
                deletions: 3,
            },
        ];
        let data = build_project_data("proj-a", "Project A", None, &commits, &[]);
        assert_eq!(data.commit_count, 1);
        assert_eq!(data.lines_added, 10);
        assert_eq!(data.lines_removed, 5);
    }

    #[test]
    fn test_build_project_data_counts_proposals() {
        let proposals = vec![
            ProposalEntry {
                slug: "a".to_string(),
                date: "2024-01-01".to_string(),
                project: "proj".to_string(),
                description: "".to_string(),
                task_count: 1,
            },
            ProposalEntry {
                slug: "b".to_string(),
                date: "2024-02-01".to_string(),
                project: "other".to_string(),
                description: "".to_string(),
                task_count: 2,
            },
        ];
        let data = build_project_data("proj", "Proj", None, &[], &proposals);
        assert_eq!(data.proposal_count, 1);
    }

    // ========================================================================
    // collect_test_metrics() integration tests — coverage decoupled from framework
    // ========================================================================

    fn make_config_single_project(
        name: &str,
        path: &str,
        coverage_command: Option<&str>,
        coverage_result_path: Option<&str>,
    ) -> Config {
        Config {
            title: None,
            output: None,
            projects: vec![ProjectConfig {
                name: name.to_string(),
                path: path.to_string(),
                description: None,
                branch: None,
                coverage_command: coverage_command.map(|s| s.to_string()),
                coverage_result_path: coverage_result_path.map(|s| s.to_string()),
                url: None,
            }],
            filters: None,
        }
    }

    #[test]
    fn test_collect_metrics_phase1_runs_coverage_command_without_framework() {
        // Phase 1 test: coverage_command generates a file; no framework markers.
        // If Phase 1 gate were still present, command wouldn't run → file missing → None.
        let dir = tempfile::tempdir().unwrap();
        // Use a relative path so the shell command works cross-platform
        // (current_dir is set to project path by collect_test_metrics)
        let cmd = "echo '<coverage line-rate=\"0.75\"></coverage>' > coverage.xml";
        let config = make_config_single_project(
            "test-proj",
            dir.path().to_str().unwrap(),
            Some(cmd),
            Some("coverage.xml"),
        );
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "none");
        assert_eq!(
            metrics[0].coverage_percent,
            Some(75.0),
            "coverage_command must run even when framework is 'none'"
        );
        assert_eq!(metrics[0].test_file_count, 0);
        assert_eq!(metrics[0].test_case_count, 0);
    }

    #[test]
    fn test_collect_metrics_phase3_discovers_preexisting_istanbul_json() {
        // Phase 3 test: pre-existing Istanbul JSON, no framework.
        let dir = tempfile::tempdir().unwrap();
        let cov_dir = dir.path().join("coverage");
        std::fs::create_dir(&cov_dir).unwrap();
        std::fs::write(
            cov_dir.join("coverage-summary.json"),
            r#"{"total":{"lines":{"pct":88.5}}}"#,
        )
        .unwrap();
        let config = make_config_single_project(
            "istanbul-proj",
            dir.path().to_str().unwrap(),
            None,
            Some("coverage/coverage-summary.json"),
        );
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "none");
        assert_eq!(metrics[0].coverage_percent, Some(88.5));
        assert_eq!(metrics[0].test_file_count, 0);
        assert_eq!(metrics[0].test_case_count, 0);
    }

    #[test]
    fn test_collect_metrics_phase3_discovers_preexisting_cobertura_xml() {
        // Phase 3 test: pre-existing Cobertura XML via explicit path, no framework.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("cov.xml"),
            r#"<coverage line-rate="0.91"></coverage>"#,
        )
        .unwrap();
        let config = make_config_single_project(
            "cobertura-proj",
            dir.path().to_str().unwrap(),
            None,
            Some("cov.xml"),
        );
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "none");
        assert_eq!(metrics[0].coverage_percent, Some(91.0));
    }

    #[test]
    fn test_collect_metrics_no_framework_no_coverage_file() {
        // No framework, no coverage file → coverage_percent is None.
        let dir = tempfile::tempdir().unwrap();
        let config =
            make_config_single_project("empty-proj", dir.path().to_str().unwrap(), None, None);
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "none");
        assert_eq!(metrics[0].coverage_percent, None);
        assert_eq!(metrics[0].test_file_count, 0);
        assert_eq!(metrics[0].test_case_count, 0);
    }

    #[test]
    fn test_collect_metrics_coverage_command_fails_no_output() {
        // coverage_command exits non-zero, produces no file → coverage_percent is None.
        let dir = tempfile::tempdir().unwrap();
        let config = make_config_single_project(
            "fail-proj",
            dir.path().to_str().unwrap(),
            Some("exit 1"),
            Some("coverage.xml"),
        );
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "none");
        assert_eq!(metrics[0].coverage_percent, None);
    }

    #[test]
    fn test_collect_metrics_auto_discovery_without_explicit_path() {
        // framework=none, no coverage_result_path, but coverage.xml exists → auto-discovered.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("coverage.xml"),
            r#"<coverage line-rate="0.64"></coverage>"#,
        )
        .unwrap();
        let config =
            make_config_single_project("auto-proj", dir.path().to_str().unwrap(), None, None);
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "none");
        assert_eq!(
            metrics[0].coverage_percent,
            Some(64.0),
            "auto-discovery must work even for unknown frameworks"
        );
    }

    #[test]
    fn test_collect_metrics_known_framework_regression() {
        // Regression test: Cargo project with coverage must still work after Phase 3 refactor.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(test)] mod tests { #[test] fn it_works() {} }",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("coverage.xml"),
            r#"<coverage line-rate="0.82"></coverage>"#,
        )
        .unwrap();
        let config =
            make_config_single_project("cargo-proj", dir.path().to_str().unwrap(), None, None);
        let metrics = collect_test_metrics(&config);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].framework, "cargo test");
        assert_eq!(metrics[0].coverage_percent, Some(82.0));
        assert!(metrics[0].test_file_count >= 1, "should find test files");
        assert!(metrics[0].test_case_count >= 1, "should count test cases");
    }
}
