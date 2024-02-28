use crate::{
    error::ApiError,
    templates::{TodoListModel, TodoModel},
    CustomError,
};
use askama::Template;
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Form, Json,
};
use axum_extra::extract::WithRejection;
use piapi_rs::types::todo::TodoItem;
use thiserror::Error;

pub(crate) trait TodoApiController {}

async fn get_todos_inner(pool: &sqlx::SqlitePool) -> Result<Vec<TodoItem>, sqlx::Error> {
    sqlx::query_as::<_, DbTodo>("SELECT * FROM todos")
        .fetch_all(pool)
        .await
        .map(|todos| {
            todos
                .into_iter()
                .map(|todo| TodoItem {
                    id: todo.id as u64,
                    name: todo.name,
                    done: todo.done,
                })
                .collect::<Vec<_>>()
        })
}

pub(crate) async fn get_all_todos(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<(StatusCode, Json<Todos>), CustomError> {
    let rows = sqlx::query_as::<_, DbTodo>("SELECT * FROM todos")
        .fetch_all(&pool)
        .await
        .map_err(|e| CustomError::TableMissing(e.to_string()))?;

    Ok((StatusCode::OK, Json(Todos { items: rows })))
}

pub(crate) async fn get_todos_list(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, ApiError> {
    let todos = get_todos_inner(&pool).await?;
    TodoListModel {
        todos,
        filter: "All".to_string(),
    }
    .render()
    .map_err(|e| e.into())
}

async fn read_todo_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<DbTodo, ApiError> {
    sqlx::query_as::<_, DbTodo>(
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

#[derive(serde::Serialize)]
pub(crate) struct Todos {
    items: Vec<DbTodo>,
}

#[derive(sqlx::FromRow, serde::Serialize, Default)]
pub(crate) struct DbTodo {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) done: bool,
}

impl From<DbTodo> for TodoItem {
    fn from(value: DbTodo) -> Self {
        TodoItem {
            id: value.id as u64,
            name: value.name,
            done: value.done,
        }
    }
}

impl From<(i64, String, bool)> for DbTodo {
    fn from(value: (i64, String, bool)) -> Self {
        Self {
            id: value.0,
            name: value.1,
            done: value.2,
        }
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct CreateTodo {
    name: String,
}

#[derive(Debug, Error)]
pub(crate) enum JsonHandlerError {
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
}

impl IntoResponse for JsonHandlerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            JsonHandlerError::JsonExtractorRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
        };

        tracing::error!("{}: {}", status, message);

        let payload = serde_json::json!({
            "message": message,
            "origin": "with_rejection"
        });

        (status, Json(payload)).into_response()
    }
}

async fn create_todo_inner(
    payload: &CreateTodo,
    pool: &sqlx::SqlitePool,
) -> Result<i64, sqlx::Error> {
    sqlx::query("INSERT INTO todos (name) Values (?1);")
        .bind(&payload.name)
        .execute(pool)
        .await
        .map(|result| result.last_insert_rowid())
}

pub(crate) async fn create_todo_from_form(
    State(pool): State<sqlx::SqlitePool>,
    Form(body): Form<CreateTodo>,
) -> Result<impl IntoResponse, ApiError> {
    let id = create_todo_inner(&body, &pool).await?;
    let todo = read_todo_by_id(&pool, id).await?;
    let template = TodoModel {
        todo: TodoItem {
            id: todo.id as u64,
            name: todo.name,
            done: todo.done,
        },
    };

    Ok(template)
}

pub(crate) async fn create_todo(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateTodo>,
) -> (StatusCode, Json<DbTodo>) {
    // insert your application logic here

    match sqlx::query("INSERT INTO todos (name, done) Values (?1, 0);")
        .bind(&payload.name)
        .execute(&pool)
        .await
    {
        Ok(query_result) => {
            let id = query_result.last_insert_rowid();
            (
                StatusCode::CREATED,
                Json(DbTodo {
                    id: id.try_into().unwrap(),
                    name: payload.name,
                    done: false,
                }),
            )
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json::default()),
    }
}

pub(crate) async fn read_todo(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> (StatusCode, Json<DbTodo>) {
    // insert your application logic here

    let failure = (StatusCode::INTERNAL_SERVER_ERROR, Json::default());

    if let Ok(todo) = read_todo_by_id(&pool, id).await {
        (StatusCode::OK, Json(todo))
    } else {
        failure
    }
}

#[derive(serde::Deserialize, Default, Debug)]
pub(crate) struct UpdateTodo {
    name: String,
    done: bool,
}

pub(crate) async fn update_todo(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateTodo>, JsonHandlerError>,
) -> (StatusCode, Json<DbTodo>) {
    // insert your application logic here

    let failure = (StatusCode::INTERNAL_SERVER_ERROR, Json::default());

    match sqlx::query(
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
    .execute(&pool)
    .await
    {
        Ok(query_result) => {
            if let Ok(todo) = read_todo_by_id(&pool, id).await {
                (StatusCode::OK, Json(todo))
            } else {
                tracing::error!("Failed to read back todo after update!");
                failure
            }
        }
        Err(e) => {
            tracing::error!("{}", e);
            failure
        }
    }
}

pub(crate) async fn delete_todo(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> StatusCode {
    // insert your application logic here

    match sqlx::query(
        r#"
            DELETE FROM todos
            WHERE id = (?1)
        ;
        "#,
    )
    .bind(id)
    .execute(&pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub(crate) async fn toggle_todo_state(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, ApiError> {
    // insert your application logic here

    let mut todo = read_todo_by_id(&pool, id).await?;
    todo.done = !todo.done;
    match sqlx::query(
        r#"
        UPDATE todos
        SET (done) = (?2)
        WHERE id = (?1);
        "#,
    )
    .bind(&id)
    .bind(todo.done)
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(TodoModel { todo: todo.into() }),
        Err(e) => {
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}

enum TodosFilter {
    All,
    Active,
    Completed,
}

impl TryFrom<String> for TodosFilter {
    type Error = ApiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "all" => Ok(Self::All),
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            _ => Err(ApiError::FilterError),
        }
    }
}

impl TodosFilter {
    pub async fn select(&self, pool: &sqlx::SqlitePool) -> Result<Vec<TodoItem>, sqlx::Error> {
        fn create_query(filter: &TodosFilter) -> &str {
            match filter {
                TodosFilter::All => "SELECT * FROM todos",
                TodosFilter::Active => "SELECT * FROM todos WHERE done = 0",
                TodosFilter::Completed => "SELECT * FROM todos WHERE done = 1",
            }
        }

        sqlx::query_as::<_, DbTodo>(create_query(&self))
            .fetch_all(pool)
            .await
            .map(|todos| todos.into_iter().map(|todo| todo.into()).collect())
    }
}

pub(crate) async fn get_filtered_todos(
    Path(filter): Path<String>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, ApiError> {
    let filter_parsed: TodosFilter = filter.clone().try_into()?;
    let todos = filter_parsed.select(&pool).await?;

    Ok(TodoListModel { todos, filter })
}
