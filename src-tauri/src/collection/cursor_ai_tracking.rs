use super::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ScoredCommit {
    pub commit_hash: String,
    pub branch_name: String,
    pub lines_added: i64,
    pub lines_deleted: i64,
    pub tab_lines_added: i64,
    pub tab_lines_deleted: i64,
    pub composer_lines_added: i64,
    pub composer_lines_deleted: i64,
    pub human_lines_added: i64,
    pub human_lines_deleted: i64,
    pub blank_lines_added: i64,
    pub blank_lines_deleted: i64,
    pub commit_message: String,
    pub commit_date: String,
    pub ai_percentage: f64,
}

#[derive(Debug, Serialize)]
pub(super) struct ScoredCommitsResponse {
    pub items: Vec<ScoredCommit>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize)]
pub(super) struct AiTrackingStats {
    pub total_commits: u64,
    pub total_lines_added: i64,
    pub total_lines_deleted: i64,
    pub total_ai_lines_added: i64,
    pub total_ai_lines_deleted: i64,
    pub total_human_lines_added: i64,
    pub total_human_lines_deleted: i64,
    pub avg_ai_percentage: f64,
    pub model_distribution: Vec<ModelStat>,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelStat {
    pub model: String,
    pub code_count: u64,
}

fn tracking_db_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cursor").join("ai-tracking").join("ai-code-tracking.db"))
}

fn open_tracking_db() -> Result<Connection, String> {
    let path = tracking_db_path()
        .ok_or_else(|| "cannot determine home directory for cursor tracking db".to_string())?;
    if !path.exists() {
        return Err(format!(
            "cursor tracking db not found at {}",
            path.display()
        ));
    }
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    Connection::open_with_flags(&path, flags)
        .map_err(|e| format!("failed to open cursor tracking db: {e:?}"))
}

pub(super) fn query_scored_commits(
    page: u32,
    page_size: u32,
) -> Result<ScoredCommitsResponse, String> {
    let db = open_tracking_db()?;
    let offset = ((page.max(1) - 1) * page_size) as i64;
    let limit = page_size.clamp(1, 200) as i64;

    let total: u64 = db
        .query_row("SELECT COUNT(1) FROM scored_commits", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| format!("failed to count scored_commits: {e:?}"))? as u64;

    let mut stmt = db
        .prepare(
            "SELECT commitHash, branchName,
                    linesAdded, linesDeleted,
                    tabLinesAdded, tabLinesDeleted,
                    composerLinesAdded, composerLinesDeleted,
                    humanLinesAdded, humanLinesDeleted,
                    blankLinesAdded, blankLinesDeleted,
                    commitMessage, commitDate, v2AiPercentage
             FROM scored_commits
             ORDER BY scoredAt DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("failed to prepare scored_commits query: {e:?}"))?;

    let rows = stmt
        .query_map(params![limit, offset], |row| {
            let ai_pct_str: String = row.get(14)?;
            let ai_percentage = ai_pct_str.parse::<f64>().unwrap_or(0.0);
            Ok(ScoredCommit {
                commit_hash: row.get(0)?,
                branch_name: row.get(1)?,
                lines_added: row.get(2)?,
                lines_deleted: row.get(3)?,
                tab_lines_added: row.get(4)?,
                tab_lines_deleted: row.get(5)?,
                composer_lines_added: row.get(6)?,
                composer_lines_deleted: row.get(7)?,
                human_lines_added: row.get(8)?,
                human_lines_deleted: row.get(9)?,
                blank_lines_added: row.get(10)?,
                blank_lines_deleted: row.get(11)?,
                commit_message: row.get(12)?,
                commit_date: row.get(13)?,
                ai_percentage,
            })
        })
        .map_err(|e| format!("failed to query scored_commits: {e:?}"))?;

    let items: Vec<ScoredCommit> = rows.flatten().collect();

    Ok(ScoredCommitsResponse {
        items,
        total,
        page: page.max(1),
        page_size: page_size.clamp(1, 200),
    })
}

pub(super) fn query_ai_code_stats() -> Result<AiTrackingStats, String> {
    let db = open_tracking_db()?;

    let (total_commits, total_lines_added, total_lines_deleted, total_ai_lines_added, total_ai_lines_deleted, total_human_lines_added, total_human_lines_deleted, avg_ai_pct) = db
        .query_row(
            "SELECT
                COUNT(1),
                COALESCE(SUM(linesAdded), 0),
                COALESCE(SUM(linesDeleted), 0),
                COALESCE(SUM(tabLinesAdded + composerLinesAdded), 0),
                COALESCE(SUM(tabLinesDeleted + composerLinesDeleted), 0),
                COALESCE(SUM(humanLinesAdded), 0),
                COALESCE(SUM(humanLinesDeleted), 0),
                COALESCE(AVG(CAST(v2AiPercentage AS REAL)), 0.0)
             FROM scored_commits",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)? as u64,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, f64>(7)?,
                ))
            },
        )
        .map_err(|e| format!("failed to query ai tracking stats: {e:?}"))?;

    let mut model_stmt = db
        .prepare(
            "SELECT COALESCE(model, 'unknown') AS m, COUNT(1) AS cnt
             FROM ai_code_hashes
             WHERE source != 'human'
             GROUP BY m
             ORDER BY cnt DESC",
        )
        .map_err(|e| format!("failed to prepare model distribution query: {e:?}"))?;

    let model_rows = model_stmt
        .query_map([], |row| {
            Ok(ModelStat {
                model: row.get(0)?,
                code_count: row.get::<_, i64>(1)? as u64,
            })
        })
        .map_err(|e| format!("failed to query model distribution: {e:?}"))?;

    let model_distribution: Vec<ModelStat> = model_rows.flatten().collect();

    Ok(AiTrackingStats {
        total_commits,
        total_lines_added,
        total_lines_deleted,
        total_ai_lines_added,
        total_ai_lines_deleted,
        total_human_lines_added,
        total_human_lines_deleted,
        avg_ai_percentage: avg_ai_pct,
        model_distribution,
    })
}
