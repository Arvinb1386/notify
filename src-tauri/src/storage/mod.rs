use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::notifications::dumpsys_parser::{NotificationItem, NotificationStatus};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::DatabaseError(format!("Failed to open DB: {}", e)))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notifications (
                id TEXT PRIMARY KEY,
                package_name TEXT NOT NULL,
                app_name TEXT,
                title TEXT,
                body TEXT,
                subtext TEXT,
                channel_id TEXT,
                post_time INTEGER NOT NULL,
                is_otp INTEGER NOT NULL DEFAULT 0,
                otp_code TEXT,
                fingerprint TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_post_time ON notifications(post_time DESC);
            CREATE INDEX IF NOT EXISTS idx_package ON notifications(package_name);
            ",
        )
        .map_err(|e| AppError::DatabaseError(format!("Migration failed: {}", e)))?;

        info!("Database migrations executed successfully");
        Ok(())
    }

    pub fn insert_notification(&self, item: &NotificationItem) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO notifications (
                id, package_name, app_name, title, body, subtext, channel_id, post_time, is_otp, otp_code, fingerprint
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                item.id,
                item.package_name,
                item.app_name,
                item.title,
                item.body,
                item.subtext,
                item.channel_id,
                item.post_time,
                if item.is_otp { 1 } else { 0 },
                item.otp_code,
                item.fingerprint
            ],
        )
        .map_err(|e| AppError::DatabaseError(format!("Insert notification failed: {}", e)))?;

        Ok(())
    }

    pub fn get_recent_notifications(&self, limit: u32) -> AppResult<Vec<NotificationItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, package_name, app_name, title, body, subtext, channel_id, post_time, is_otp, otp_code, fingerprint
                 FROM notifications
                 ORDER BY post_time DESC
                 LIMIT ?1",
            )
            .map_err(|e| AppError::DatabaseError(format!("Query prep failed: {}", e)))?;

        let rows = stmt
            .query_map([limit], |row| {
                let is_otp_int: i32 = row.get(8)?;
                Ok(NotificationItem {
                    id: row.get(0)?,
                    package_name: row.get(1)?,
                    app_name: row.get(2)?,
                    title: row.get(3)?,
                    body: row.get(4)?,
                    subtext: row.get(5)?,
                    channel_id: row.get(6)?,
                    post_time: row.get(7)?,
                    is_otp: is_otp_int == 1,
                    otp_code: row.get(9)?,
                    status: NotificationStatus::Posted,
                    fingerprint: row.get(10)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(format!("Query exec failed: {}", e)))?;

        let mut items = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                items.push(item);
            }
        }

        Ok(items)
    }

    pub fn clear_notifications(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notifications", [])
            .map_err(|e| AppError::DatabaseError(format!("Clear history failed: {}", e)))?;
        Ok(())
    }
}
