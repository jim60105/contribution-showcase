use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use chrono::Local;

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
    if j < bytes.len() && bytes[j] == b'(' {
        if let Some(close) = subject[j..].find(')') {
            scope = subject[j + 1..j + close].to_string();
            j += close + 1;
        }
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
        if date.len() != 10
            || date.as_bytes()[4] != b'-'
            || date.as_bytes()[7] != b'-'
        {
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
            if let Some(ref author) = filters.author {
                if !c.subject.is_empty()
                    && !c.project.is_empty()
                {
                    // We need to re-check author from the git log.
                    // Actually, author info is not in CommitEntry. We need to filter during collection.
                    // For now, this filter is applied during collection in a wrapper.
                    // But we stored author filtering needs to happen at collection time.
                    // Let's keep this as a pass-through; author filtering happens in collect().
                    let _ = author;
                }
            }
            if let Some(ref since) = filters.since {
                if c.date < *since {
                    return false;
                }
            }
            if let Some(ref until) = filters.until {
                if c.date > *until {
                    return false;
                }
            }
            if let Some(ref types) = filters.types {
                if !types.contains(&c.commit_type) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    let filtered_proposals: Vec<ProposalEntry> = proposals
        .iter()
        .filter(|p| {
            if let Some(ref since) = filters.since {
                if p.date < *since {
                    return false;
                }
            }
            if let Some(ref until) = filters.until {
                if p.date > *until {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    (filtered_commits, filtered_proposals)
}

fn collect_git_commits_filtered(
    repo_path: &str,
    project_name: &str,
    author_filter: Option<&str>,
) -> Result<Vec<CommitEntry>> {
    let path = Path::new(repo_path);
    if !path.exists() || !path.join(".git").exists() {
        eprintln!("Warning: skipping '{}' — not a git repository", repo_path);
        return Ok(Vec::new());
    }

    let mut args = vec![
        "-C".to_string(),
        repo_path.to_string(),
        "log".to_string(),
        "--all".to_string(),
        format!("--format={}%H|||%aN|||%aI|||%s", COMMIT_DELIM),
        "--shortstat".to_string(),
    ];

    if let Some(author) = author_filter {
        args.push(format!("--author={}", author));
    }

    let output = Command::new("git").args(&args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Warning: git log failed for '{}': {}", repo_path, stderr);
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
                let (commit_type, scope) = parse_conventional_commit(parts[3]);
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
            if trimmed.is_empty() {
                continue;
            }
            for part in trimmed.split(',') {
                let part = part.trim();
                if part.contains("insertion") {
                    if let Some(n) = part.split_whitespace().next() {
                        c.insertions = n.parse().unwrap_or(0);
                    }
                } else if part.contains("deletion") {
                    if let Some(n) = part.split_whitespace().next() {
                        c.deletions = n.parse().unwrap_or(0);
                    }
                }
            }
        }
    }
    if let Some(c) = current.take() {
        commits.push(c);
    }

    Ok(commits)
}

fn build_timeline(commits: &[CommitEntry]) -> Vec<TimelineEntry> {
    let mut week_counts: HashMap<String, usize> = HashMap::new();

    for c in commits {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&c.date, "%Y-%m-%d") {
            let iso_week = date.format("%G-W%V").to_string();
            *week_counts.entry(iso_week).or_insert(0) += 1;
        }
    }

    let mut weeks: Vec<(String, usize)> = week_counts.into_iter().collect();
    weeks.sort_by(|a, b| a.0.cmp(&b.0));

    let max_count = weeks.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);

    weeks
        .into_iter()
        .map(|(label, count)| TimelineEntry {
            label,
            count,
            height: (count as f64 / max_count as f64) * 100.0,
        })
        .collect()
}

fn build_type_breakdown(commits: &[CommitEntry]) -> Vec<TypeBreakdown> {
    let mut label_counts: HashMap<String, (String, usize)> = HashMap::new();
    for c in commits {
        let label = type_label(&c.commit_type).to_string();
        let canonical_type = match label.as_str() {
            "其他" => "other",
            _ => &c.commit_type,
        };
        let entry = label_counts
            .entry(label.clone())
            .or_insert_with(|| (canonical_type.to_string(), 0));
        entry.1 += 1;
    }

    let total = commits.len().max(1);
    let mut breakdown: Vec<TypeBreakdown> = label_counts
        .into_iter()
        .map(|(label, (commit_type, count))| TypeBreakdown {
            label,
            commit_type,
            count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();

    breakdown.sort_by(|a, b| b.count.cmp(&a.count));
    breakdown
}

fn build_project_data(
    project_name: &str,
    description: &str,
    commits: &[CommitEntry],
    proposals: &[ProposalEntry],
) -> ProjectData {
    let project_commits: Vec<&CommitEntry> =
        commits.iter().filter(|c| c.project == project_name).collect();
    let project_proposals: Vec<&ProposalEntry> =
        proposals.iter().filter(|p| p.project == project_name).collect();

    let lines_added: usize = project_commits.iter().map(|c| c.insertions).sum();
    let lines_removed: usize = project_commits.iter().map(|c| c.deletions).sum();

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for c in &project_commits {
        *type_counts.entry(c.commit_type.clone()).or_insert(0) += 1;
    }
    let total = project_commits.len().max(1);
    let mut top_types: Vec<TypeBreakdown> = type_counts
        .into_iter()
        .map(|(t, count)| TypeBreakdown {
            label: type_label(&t).to_string(),
            commit_type: t,
            count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();
    top_types.sort_by(|a, b| b.count.cmp(&a.count));
    top_types.truncate(5);

    ProjectData {
        name: project_name.to_string(),
        description: description.to_string(),
        commit_count: project_commits.len(),
        proposal_count: project_proposals.len(),
        lines_added,
        lines_removed,
        top_types,
    }
}

pub fn collect(config: &Config) -> Result<ShowcaseData> {
    let filters = config.filters();
    let author_filter = filters.author.as_deref();

    let mut all_commits = Vec::new();
    let mut all_proposals = Vec::new();

    for project in &config.projects {
        let commits = collect_git_commits_filtered(&project.path, &project.name, author_filter)?;
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
                &all_commits,
                &all_proposals,
            )
        })
        .filter(|p| p.commit_count > 0 || p.proposal_count > 0)
        .collect();

    let author = author_filter.unwrap_or("ALL").to_string();

    Ok(ShowcaseData {
        title: config.title.clone().unwrap_or_else(|| "貢獻總覽".to_string()),
        author,
        date_range,
        generated_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
        summary: Summary {
            total_commits,
            total_repos,
            total_proposals,
            lines_added,
            lines_removed,
        },
        timeline,
        type_breakdown,
        projects,
        proposals: all_proposals,
        commits: all_commits,
    })
}
