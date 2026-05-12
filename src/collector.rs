use std::collections::{HashMap, HashSet};
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
            if let Some(ref hashes) = filters.exclude_hashes {
                if hashes.iter().any(|h| h == &c.hash) {
                    return false;
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
        } else if part.contains("deletion") {
            if let Some(n) = part.split_whitespace().next() {
                deletions = n.parse().unwrap_or(0);
            }
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

    let mut args = vec![
        "-C".to_string(),
        repo_path.to_string(),
        "log".to_string(),
    ];

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
    let mut week_lines: HashMap<String, usize> = HashMap::new();

    for c in commits {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&c.date, "%Y-%m-%d") {
            let iso_week = date.format("%G-W%V").to_string();
            *week_lines.entry(iso_week).or_insert(0) += c.insertions + c.deletions;
        }
    }

    let mut weeks: Vec<(String, usize)> = week_lines.into_iter().collect();
    weeks.sort_by(|a, b| a.0.cmp(&b.0));

    let max_lines = weeks.iter().map(|(_, l)| *l).max().unwrap_or(0);

    weeks
        .into_iter()
        .map(|(label, lines)| TimelineEntry {
            label,
            lines,
            height: if max_lines == 0 {
                0.0
            } else {
                (lines as f64 / max_lines as f64) * 100.0
            },
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

    breakdown.sort_by(|a, b| b.lines.cmp(&a.lines));
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

    let mut type_lines: HashMap<String, usize> = HashMap::new();
    for c in &project_commits {
        *type_lines.entry(c.commit_type.clone()).or_insert(0) += c.insertions + c.deletions;
    }
    let total_lines: usize = project_commits.iter().map(|c| c.insertions + c.deletions).sum();
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
    top_types.sort_by(|a, b| b.lines.cmp(&a.lines));
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
            avg_daily_lines,
        },
        timeline,
        type_breakdown,
        projects,
        proposals: all_proposals,
        commits: all_commits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(other_entries.len(), 1, "Should have exactly one '其他' entry");
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
        let commits = vec![
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
        let commits = vec![
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
        ];
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
            assert_eq!(entry.height, 0.0, "Height should be 0.0 when all lines are zero");
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
            assert_eq!(entry.percentage, 0.0, "Percentage should be 0.0 when all lines are zero");
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
        assert_eq!(breakdown[1].lines, 40);  // docs: 30+10
        assert_eq!(breakdown[2].lines, 7);   // fix: 5+2
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
        let data = build_project_data("proj", "Test project", &commits, &[]);
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
        let data = build_project_data("proj", "Test project", &commits, &[]);
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
        let (t, s) = apply_wiki_override("VMS.wiki", "feat(nav): add sidebar");
        assert_eq!(t, "docs");
        assert_eq!(s, "nav");
    }

    #[test]
    fn test_wiki_override_non_conventional_becomes_docs() {
        let (t, s) = apply_wiki_override("VMS.wiki", "Update architecture page");
        assert_eq!(t, "docs");
        assert_eq!(s, "");
    }

    #[test]
    fn test_wiki_override_already_docs_stays_docs() {
        let (t, s) = apply_wiki_override("VMS.wiki", "docs: update readme");
        assert_eq!(t, "docs");
        assert_eq!(s, "");
    }

    #[test]
    fn test_non_wiki_project_not_overridden() {
        let (t, _) = apply_wiki_override("VMS-Frontend", "feat: add map");
        assert_eq!(t, "feat");
    }

    #[test]
    fn test_contains_wiki_but_not_suffix_not_overridden() {
        let (t, _) = apply_wiki_override("VMS.wiki-tools", "feat: add parser");
        assert_eq!(t, "feat");
    }

    #[test]
    fn test_uppercase_wiki_not_overridden() {
        let (t, _) = apply_wiki_override("VMS.WIKI", "feat: add page");
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
                exclude_hashes: Some(vec![]),
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 1, "Empty exclude list should retain all commits");
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
        let commits = vec![
            CommitEntry {
                hash: "dd33ee63950bb49a284de835528343561f1a70d5".to_string(),
                date: "2024-01-01".to_string(),
                commit_type: "feat".to_string(),
                scope: String::new(),
                subject: "feat: a".to_string(),
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
                exclude_hashes: Some(vec!["dd33ee63".to_string()]),
            }),
        };
        let (filtered, _) = apply_filters(&commits, &proposals, &config);
        assert_eq!(filtered.len(), 1, "Short prefix should not match full hash");
    }
}
