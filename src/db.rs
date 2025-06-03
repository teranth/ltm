use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::home_dir;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::Row;
use std::str::FromStr;

use crate::models::{Comment, ProjectSummary, Ticket};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = home_dir()
            .context("Could not find home directory")?
            .join(".ltm")
            .join("tickets.db");

        std::fs::create_dir_all(db_path.parent().unwrap())?;

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run migrations")?;

        Ok(Self { pool })
    }

    // Helper methods for testing
    #[allow(dead_code)]
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn init_db(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tickets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            );

            CREATE TABLE IF NOT EXISTS comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticket_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (ticket_id) REFERENCES tickets(id)
            );

            CREATE TABLE IF NOT EXISTS time_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticket_id INTEGER NOT NULL,
                hours INTEGER NOT NULL,
                minutes INTEGER NOT NULL,
                started_at DATETIME,
                ended_at DATETIME,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (ticket_id) REFERENCES tickets(id)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_ticket(&self, project: &str, name: &str, description: &str) -> Result<i64> {
        let now = Utc::now().naive_utc();
        let id = sqlx::query(
            r#"
            INSERT INTO tickets (project, name, description, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(project)
        .bind(name)
        .bind(description)
        .bind("open")
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn get_ticket(&self, id: i64) -> Result<Option<Ticket>> {
        let ticket = sqlx::query_as::<_, Ticket>(
            "SELECT id, project, name, description, status, created_at, updated_at FROM tickets WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ticket)
    }

    pub async fn list_tickets(&self, project: Option<&str>) -> Result<Vec<Ticket>> {
        let tickets = if let Some(project) = project {
            sqlx::query_as::<_, Ticket>(
                "SELECT id, project, name, description, status, created_at, updated_at FROM tickets WHERE project = ? ORDER BY created_at DESC"
            )
            .bind(project)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Ticket>(
                "SELECT id, project, name, description, status, created_at, updated_at FROM tickets ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(tickets)
    }

    pub async fn update_ticket_status(&self, id: i64, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tickets SET status = ?, updated_at = ? WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_ticket(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM tickets WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_comment(&self, ticket_id: i64, content: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO comments (ticket_id, content, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(ticket_id)
        .bind(content)
        .bind(Utc::now().naive_utc())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_comments(&self, ticket_id: i64) -> Result<Vec<Comment>> {
        let comments = sqlx::query_as::<_, Comment>(
            "SELECT id, ticket_id, content, created_at FROM comments WHERE ticket_id = ? ORDER BY created_at DESC"
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    pub async fn add_time_log(
        &self,
        ticket_id: i64,
        hours: i32,
        minutes: i32,
        started_at: Option<DateTime<Utc>>,
        ended_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO time_logs (ticket_id, hours, minutes, started_at, ended_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ticket_id)
        .bind(hours)
        .bind(minutes)
        .bind(started_at.map(|dt| dt.naive_utc()))
        .bind(ended_at.map(|dt| dt.naive_utc()))
        .bind(Utc::now().naive_utc())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_project_summary(&self, project: &str) -> Result<ProjectSummary> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_tickets,
                SUM(CASE WHEN status = 'open' THEN 1 ELSE 0 END) as open_tickets,
                SUM(CASE WHEN status = 'closed' THEN 1 ELSE 0 END) as closed_tickets,
                COALESCE(SUM(tl.hours + tl.minutes / 60.0), 0.0) as total_time_hours
            FROM tickets t
            LEFT JOIN time_logs tl ON t.id = tl.ticket_id
            WHERE t.project = ?
            GROUP BY t.project
            "#
        )
        .bind(project)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                Ok(ProjectSummary {
                    project: project.to_string(),
                    total_tickets: row.get::<i64, _>(0),
                    open_tickets: row.get::<Option<i64>, _>(1).unwrap_or(0),
                    closed_tickets: row.get::<Option<i64>, _>(2).unwrap_or(0),
                    total_time_hours: row.get::<f64, _>(3),
                })
            }
            None => {
                // Return empty summary for non-existent projects
                Ok(ProjectSummary {
                    project: project.to_string(),
                    total_tickets: 0,
                    open_tickets: 0,
                    closed_tickets: 0,
                    total_time_hours: 0.0,
                })
            }
        }
    }

    pub async fn get_time_logs(&self, ticket_id: i64) -> Result<Vec<crate::models::TimeLog>> {
        let rows = sqlx::query(
            r#"
            SELECT id, ticket_id, hours, minutes, started_at, ended_at, created_at 
            FROM time_logs WHERE ticket_id = ? ORDER BY created_at DESC
            "#
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await?;

        let mut time_logs = Vec::new();
        for row in rows {
            time_logs.push(crate::models::TimeLog {
                id: row.get("id"),
                ticket_id: row.get("ticket_id"),
                hours: row.get("hours"),
                minutes: row.get("minutes"),
                started_at: row.get("started_at"),
                ended_at: row.get("ended_at"),
                created_at: row.get("created_at"),
            });
        }

        Ok(time_logs)
    }
} 