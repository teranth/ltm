use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Ticket {
    pub id: i64,
    pub project: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Comment {
    pub id: i64,
    pub ticket_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TimeLog {
    pub id: i64,
    pub ticket_id: i64,
    pub hours: i32,
    pub minutes: i32,
    pub started_at: Option<NaiveDateTime>,
    pub ended_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub project: String,
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub closed_tickets: i64,
    pub total_time_hours: f64,
} 