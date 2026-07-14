use serde::{Deserialize, Serialize};

/// Bir görevin Kanban akışındaki yerini temsil eder.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

/// Frontend'e gönderilen temel görev modeli.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub status: TaskStatus,
    pub scheduled_for: Option<String>,
}

impl Task {
    pub fn new(id: i64, title: String, scheduled_for: Option<String>) -> Result<Self, String> {
        let title = title.trim().to_owned();

        if title.is_empty() {
            return Err("Görev başlığı boş olamaz.".into());
        }

        if title.chars().count() > 120 {
            return Err("Görev başlığı 120 karakterden uzun olamaz.".into());
        }

        Ok(Self {
            id,
            title,
            status: TaskStatus::Todo,
            scheduled_for,
        })
    }
}

impl TaskStatus {
    /// Enum değerini veritabanında okunabilir ve kararlı bir metin olarak saklarız.
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }

    /// Veritabanındaki metni tekrar güçlü tipli Rust enum'una dönüştürür.
    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "todo" => Ok(Self::Todo),
            "in_progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            other => Err(format!("Bilinmeyen görev durumu: {other}")),
        }
    }
}
