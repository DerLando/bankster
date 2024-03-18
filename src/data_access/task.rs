use chrono::Utc;
use sqlx::Row;

use crate::data::{TableRow, TodoRow};

use super::utils::DateTime;

pub(crate) type TaskRow = (i64, String, String, DateTime, DateTime, bool);
pub(crate) type TaskRowInput = (String, String, DateTime, bool);

pub(crate) async fn select_row(id: i64, pool: &sqlx::SqlitePool) -> Result<TaskRow, sqlx::Error> {
    sqlx::query(
        r#"
            SELECT * FROM tasks
            WHERE id = (?1)
        ;
        "#,
    )
    .bind(id)
    .map(|row: sqlx::sqlite::SqliteRow| {
        (
            row.get("id"),
            row.get("name"),
            row.get("description"),
            row.get("created"),
            row.get("due"),
            row.get("done"),
        )
    })
    .fetch_one(pool)
    .await
}

pub(crate) async fn insert_row(
    input: TaskRowInput,
    pool: &sqlx::SqlitePool,
) -> Result<i64, sqlx::Error> {
    let (name, description, due, done) = input;
    sqlx::query(
        r#"
          INSERT INTO tasks (name, description, created, due, done)
        VALUES (?1, ?2, ?3, ?4, ?5);  
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(Utc::now())
    .bind(due)
    .bind(done)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

pub(crate) async fn query_todos_by_id(
    task_id: i64,
    pool: &sqlx::SqlitePool,
) -> Result<Vec<<TodoRow as TableRow>::Data>, sqlx::Error> {
    sqlx::query(
        r#"
          SELECT * FROM todos 
          JOIN tasktodos tt ON tt.task_id = (?1) AND tt.todo_id = id
        ;
        "#,
    )
    .bind(task_id)
    .map(|row: sqlx::sqlite::SqliteRow| (row.get("id"), row.get("name"), row.get("done")))
    .fetch_all(pool)
    .await
}
