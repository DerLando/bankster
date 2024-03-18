use crate::{
    api::types::task::{CreateTask, TaskHeader, TaskModel},
    data_access,
};

pub async fn get_all_headers(pool: &sqlx::SqlitePool) -> Result<Vec<TaskHeader>, sqlx::Error> {
    sqlx::query_as::<_, TaskHeader>("SELECT id, name FROM tasks")
        .fetch_all(pool)
        .await
}

pub async fn get_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<TaskModel, sqlx::Error> {
    let task = data_access::task::select_row(id, &pool).await?;
    let (id, name, description, created, due, done) = task;
    let todos = data_access::task::query_todos_by_id(id, &pool)
        .await?
        .into_iter()
        .map(|values| values.into())
        .collect::<Vec<_>>();

    Ok(TaskModel {
        id,
        name,
        description,
        created,
        due,
        done,
        todos,
    })
}

pub async fn create_task(
    pool: &sqlx::SqlitePool,
    payload: &CreateTask,
) -> Result<i64, sqlx::Error> {
    let timestamp = data_access::utils::now();
    let row = (payload.name.clone(), String::new(), timestamp, false);
    data_access::task::insert_row(row, pool).await
}
