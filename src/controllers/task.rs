use crate::CustomError;
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;
use thiserror::Error;

pub(crate) mod html_api {
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, State},
        routing::get,
        Router,
    };

    use crate::{
        error::ApiError,
        templates::{TaskEdit, TaskIndex, Tasks},
    };

    use super::read_task_by_id;

    pub(crate) const NEST_PREFIX: &str = "/tasks";

    pub fn router(state: sqlx::SqlitePool) -> Router<sqlx::SqlitePool> {
        Router::new()
            .with_state(state)
            .route("/", get(index))
            .route("/all", get(get_tasks))
            .route("/:id", get(get_edit_task))
    }

    async fn index() -> impl IntoResponse {
        TaskIndex
    }

    async fn get_tasks(
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let headers = super::get_all_task_headers(pool).await?;
        Ok(Tasks { tasks: headers })
    }

    async fn get_edit_task(
        Path(id): Path<i64>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let task = read_task_by_id(id, &pool).await?;

        Ok(TaskEdit { task })
    }
}

#[derive(serde::Serialize)]
pub(crate) struct TaskHeaders {
    tasks: Vec<TaskHeader>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub(crate) struct TaskHeader {
    pub id: i64,
    pub name: String,
}

#[derive(serde::Serialize)]
pub(crate) struct TaskModel {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) created: crate::data::DateTime,
    pub(crate) due: crate::data::DateTime,
    pub(crate) done: bool,
    pub(crate) todos: Vec<crate::controllers::todo::DbTodo>,
}

#[derive(serde::Deserialize)]
pub(crate) struct CreateTask {
    name: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct UpdateTask {
    name: Option<String>,
    description: Option<String>,
    due: Option<crate::data::DateTime>,
    done: Option<bool>,
}

async fn get_all_task_headers(pool: sqlx::SqlitePool) -> Result<Vec<TaskHeader>, sqlx::Error> {
    sqlx::query_as::<_, TaskHeader>("SELECT id, name FROM tasks")
        .fetch_all(&pool)
        .await
}

pub(crate) async fn get_all_tasks(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<(StatusCode, Json<Vec<TaskHeader>>), CustomError> {
    let rows = sqlx::query_as::<_, TaskHeader>("SELECT id, name FROM tasks")
        .fetch_all(&pool)
        .await
        .map_err(|e| CustomError::TableMissing(e.to_string()))?;

    Ok((StatusCode::OK, Json(rows)))
}

pub(crate) async fn create_task(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateTask>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query("INSERT INTO tasks (name, created) VALUES (?1, ?2)")
        .bind(payload.name)
        .bind(crate::data::now())
        .execute(&pool)
        .await
    {
        Ok(result) => Ok((StatusCode::OK, result.last_insert_rowid().to_string())),
        Err(e) => {
            tracing::error!("{:#?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn read_task_by_id(id: i64, pool: &sqlx::SqlitePool) -> Result<TaskModel, sqlx::Error> {
    let task = crate::data::select_task_row(id, &pool).await?;
    let (id, name, description, created, due, done) = task;
    let todos = crate::data::query_todos_by_task(id, &pool)
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

pub(crate) async fn read_task(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match read_task_by_id(id, &pool).await {
        Ok(task) => Ok((StatusCode::OK, Json(task))),
        Err(e) => {
            tracing::error!("{:#?}", e);
            Err(e.to_string())
        }
    }
}

async fn update_task_inner(
    id: i64,
    payload: UpdateTask,
    pool: &sqlx::SqlitePool,
) -> Result<(), sqlx::Error> {
    let old_task = read_task_by_id(id, &pool).await?;
    sqlx::query(
        r#"
              UPDATE tasks
              SET (name) = (?2),
                  (description) = (?3),
                  (due) = (?4),
                  (done) = (?5)
              WHERE id = (?1);  
        "#,
    )
    .bind(id)
    .bind(payload.name.unwrap_or(old_task.name))
    .bind(payload.description.unwrap_or(old_task.description))
    .bind(payload.due.unwrap_or(old_task.due))
    .bind(payload.done.unwrap_or(old_task.done))
    .execute(pool)
    .await
    .map(|_| ())
}

pub(crate) async fn update_task(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<UpdateTask>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match update_task_inner(id, payload, &pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub(crate) async fn delete_task(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query(
        r#"
            DELETE FROM tasks
            WHERE id = (?1);
        "#,
    )
    .bind(id)
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

async fn add_todo_to_task(
    task_id: i64,
    todo_id: i64,
    pool: &sqlx::SqlitePool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
            INSERT INTO tasktodos (task_id, todo_id)
            VALUES (?1, ?2);
        "#,
    )
    .bind(task_id)
    .bind(todo_id)
    .execute(pool)
    .await
    .map(|_| ())
}

pub(crate) async fn add_todo(
    Path((task_id, todo_id)): Path<(i64, i64)>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match add_todo_to_task(task_id, todo_id, &pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

async fn remove_todo_from_task(
    task_id: i64,
    todo_id: i64,
    pool: &sqlx::SqlitePool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
            DELETE FROM tasktodos
            WHERE task_id = (?1) AND todo_id = (?2)
        "#,
    )
    .bind(task_id)
    .bind(todo_id)
    .execute(pool)
    .await
    .map(|_| ())
}

pub(crate) async fn remove_todo(
    Path((task_id, todo_id)): Path<(i64, i64)>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match remove_todo_from_task(task_id, todo_id, &pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
