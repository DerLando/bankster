use tracing::info;

use crate::{
    api::types::todo::{CreateTodo, TodoItem, TodoQuery, UpdateTodo},
    error::ApiError,
};

pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<TodoItem>, sqlx::Error> {
    sqlx::query_as::<_, TodoItem>("SELECT id, name, done FROM todos")
        .fetch_all(pool)
        .await
}

pub async fn get_all_matching(
    pool: &sqlx::SqlitePool,
    query: &TodoQuery,
) -> Result<Vec<TodoItem>, sqlx::Error> {
    let mut raw_query = r#"
            SELECT id, name, done FROM todos
        "#
    .to_string();

    let where_buffer = match query.done {
        Some(true) => "\n WHERE done = 1;",
        Some(false) => "\n WHERE done = 0;",
        _ => "",
    };

    raw_query.push_str(&where_buffer);

    sqlx::query_as::<_, TodoItem>(&raw_query)
        .fetch_all(pool)
        .await
}

pub async fn get_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<TodoItem, ApiError> {
    sqlx::query_as::<_, TodoItem>(
        r#"
            SELECT * FROM todos
            WHERE id = (?1)
        ;
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.into())
}

pub async fn create(pool: &sqlx::SqlitePool, payload: &CreateTodo) -> Result<i64, sqlx::Error> {
    sqlx::query("INSERT INTO todos (name) Values (?1);")
        .bind(&payload.name)
        .execute(pool)
        .await
        .map(|result| result.last_insert_rowid())
}

pub async fn update(
    pool: &sqlx::SqlitePool,
    id: i64,
    payload: &UpdateTodo,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE todos
        Set (name) = (?2),
            (done) = (?3)
        WHERE id = (?1);
        "#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.done)
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
            DELETE FROM todos
            WHERE id = (?1)
        ;
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|_| ())
}

pub(crate) async fn toggle_state(pool: &sqlx::SqlitePool, id: i64) -> Result<(), ApiError> {
    let mut todo = get_by_id(pool, id).await?;
    todo.done = !todo.done;
    sqlx::query(
        r#"
        UPDATE todos
        SET (done) = (?2)
        WHERE id = (?1);
        "#,
    )
    .bind(&id)
    .bind(todo.done)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| e.into())
}
