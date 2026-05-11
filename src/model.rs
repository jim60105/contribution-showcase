use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ShowcaseData {
    pub title: String,
    pub author: String,
    pub date_range: String,
    pub generated_at: String,
    pub summary: Summary,
    pub timeline: Vec<TimelineEntry>,
    pub type_breakdown: Vec<TypeBreakdown>,
    pub projects: Vec<ProjectData>,
    pub proposals: Vec<ProposalEntry>,
    pub commits: Vec<CommitEntry>,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_commits: usize,
    pub total_repos: usize,
    pub total_proposals: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub avg_daily_lines: f64,
}

#[derive(Debug, Serialize)]
pub struct TimelineEntry {
    pub label: String,
    pub count: usize,
    pub height: f64,
}

#[derive(Debug, Serialize)]
pub struct TypeBreakdown {
    pub commit_type: String,
    pub label: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct ProjectData {
    pub name: String,
    pub description: String,
    pub commit_count: usize,
    pub proposal_count: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub top_types: Vec<TypeBreakdown>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub date: String,
    pub commit_type: String,
    pub scope: String,
    pub subject: String,
    pub project: String,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProposalEntry {
    pub slug: String,
    pub date: String,
    pub project: String,
    pub description: String,
    pub task_count: usize,
}
