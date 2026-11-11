use crate::engine::soils::get_standard_nbr6122_soils;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct GeoDatabase {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct ProjectHistoryEntry {
    pub id: i64,
    pub title: String,
    pub engineer: String,
    pub mode: String,
    pub primary_fs: f64,
    pub status: String,
    pub created_at: String,
}

impl GeoDatabase {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_tables()?;
        db.seed_soils_if_empty()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_tables()?;
        db.seed_soils_if_empty()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS soils_catalog (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                classification TEXT NOT NULL,
                spt_range TEXT NOT NULL,
                gamma REAL NOT NULL,
                gamma_sat REAL NOT NULL,
                cohesion REAL NOT NULL,
                friction_angle_deg REAL NOT NULL,
                sigma_adm REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                engineer TEXT NOT NULL,
                mode TEXT NOT NULL,
                primary_fs REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action TEXT NOT NULL,
                details TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    fn seed_soils_if_empty(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM soils_catalog", [], |r| r.get(0))?;

        if count == 0 {
            let soils = get_standard_nbr6122_soils();
            let mut stmt = self.conn.prepare(
                "INSERT INTO soils_catalog (name, classification, spt_range, gamma, gamma_sat, cohesion, friction_angle_deg, sigma_adm)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;

            for s in soils {
                stmt.execute(params![
                    s.name,
                    s.classification,
                    s.spt_range,
                    s.gamma,
                    s.gamma_sat,
                    s.cohesion,
                    s.friction_angle_deg,
                    s.sigma_adm,
                ])?;
            }
        }
        Ok(())
    }

    pub fn log_action(&self, action: &str, details: &str) -> Result<()> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute(
            "INSERT INTO audit_log (action, details, timestamp) VALUES (?1, ?2, ?3)",
            params![action, details, now],
        )?;
        Ok(())
    }

    pub fn record_history_entry(
        &self,
        title: &str,
        engineer: &str,
        mode: &str,
        primary_fs: f64,
        status: &str,
    ) -> Result<()> {
        let now = chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
        self.conn.execute(
            "INSERT INTO project_history (title, engineer, mode, primary_fs, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![title, engineer, mode, primary_fs, status, now],
        )?;
        Ok(())
    }

    pub fn get_recent_history(&self, limit: usize) -> Result<Vec<ProjectHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, engineer, mode, primary_fs, status, created_at 
             FROM project_history ORDER BY id DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok(ProjectHistoryEntry {
                id: r.get(0)?,
                title: r.get(1)?,
                engineer: r.get(2)?,
                mode: r.get(3)?,
                primary_fs: r.get(4)?,
                status: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }
}
