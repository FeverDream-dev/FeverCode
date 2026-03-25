use crate::error::{SearchError, SearchResult};
use crate::result::SearchResults;
use rusqlite::{Connection, params};
use std::path::Path;

pub struct SearchCache {
    conn: Connection,
    ttl_hours: u64,
}

impl SearchCache {
    pub fn new<P: AsRef<Path>>(path: P, ttl_hours: u64) -> SearchResult<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS search_cache (
                query TEXT PRIMARY KEY,
                results TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| SearchError::Cache(e.to_string()))?;

        Ok(Self { conn, ttl_hours })
    }

    pub async fn get(&self, query: &str) -> Option<SearchResults> {
        let query_str = query.to_string();

        tokio::task::block_in_place(|| {
            let mut stmt = self
                .conn
                .prepare("SELECT results, timestamp FROM search_cache WHERE query = ?1")
                .ok()?;

            let mut rows = stmt.query(params![&query_str]).ok()?;

            if let Some(row) = rows.next().ok()? {
                let json_str: String = row.get(0).ok()?;
                let timestamp: i64 = row.get(1).ok()?;

                let now = chrono::Utc::now().timestamp();
                let elapsed = now - timestamp;
                let ttl_seconds = (self.ttl_hours as i64) * 3600;

                if elapsed > ttl_seconds {
                    let _ = self.delete(&query_str);
                    return None;
                }

                let results: SearchResults = serde_json::from_str(&json_str).ok()?;
                Some(results)
            } else {
                None
            }
        })
    }

    pub async fn set(&self, query: &str, results: &SearchResults) -> SearchResult<()> {
        let query_str = query.to_string();
        let json_str =
            serde_json::to_string(results).map_err(|e| SearchError::Cache(e.to_string()))?;
        let timestamp = chrono::Utc::now().timestamp();

        tokio::task::block_in_place(|| {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO search_cache (query, results, timestamp) VALUES (?1, ?2, ?3)",
                    params![&query_str, &json_str, timestamp],
                )
                .map_err(|e| SearchError::Cache(e.to_string()))?;

            self.prune_old()?;

            Ok(())
        })
    }

    fn delete(&self, query: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM search_cache WHERE query = ?1", params![query])?;
        Ok(())
    }

    fn prune_old(&self) -> Result<(), rusqlite::Error> {
        let ttl_seconds = (self.ttl_hours as i64) * 3600;
        let cutoff = chrono::Utc::now().timestamp() - ttl_seconds;

        self.conn.execute(
            "DELETE FROM search_cache WHERE timestamp < ?1",
            params![cutoff],
        )?;

        Ok(())
    }

    pub fn clear(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute("DELETE FROM search_cache", [])?;
        Ok(())
    }
}
