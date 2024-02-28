use std::str::FromStr;

use askama::Template;
use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, Method, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_extra::response::Css;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions,
};
use templates::{LoginIndex, TodoIndex};
use tokio::io::AsyncReadExt;
use tower_http::services::{ServeDir, ServeFile};
mod config;
pub(crate) mod controllers;
mod data;
mod error;
mod templates;

const DB_NAME: &str = "sqlite://data.db";
// const DB_NAME: &str = "sqlite::memory:";

enum CustomError {
    TodoNotFound,
    TableMissing(String),
    ConfigParseError,
}

impl IntoResponse for CustomError {
    fn into_response(self) -> axum::response::Response {
        match self {
            CustomError::TodoNotFound => {
                (StatusCode::BAD_REQUEST, "Could not find Todo Item").into_response()
            }
            CustomError::TableMissing(why) => {
                (StatusCode::INTERNAL_SERVER_ERROR, why).into_response()
            }
            CustomError::ConfigParseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse config file",
            )
                .into_response(),
        }
    }
}

fn build_api_router(pool: sqlx::SqlitePool) -> Router {
    Router::new()
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .route("/todo", post(controllers::create_todo))
        .route("/todo/:id", get(controllers::read_todo))
        .route("/todo/:id", put(controllers::update_todo))
        .route("/todo/:id", delete(controllers::delete_todo))
        .route("/todo", get(controllers::get_all_todos))
        .route("/backup", get(save_backup))
        .route("/tasks", get(controllers::task::get_all_tasks))
        .route("/tasks", post(controllers::task::create_task))
        .route("/tasks/:id", get(controllers::task::read_task))
        .route("/tasks/:id", put(controllers::task::update_task))
        .route("/tasks/:id", delete(controllers::task::delete_task))
        .route("/tasks/:id/todos/:id", post(controllers::task::add_todo))
        .route(
            "/tasks/:id/todos/:id",
            delete(controllers::task::remove_todo),
        )
        .with_state(pool)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = read_config().await;

    let sqlite_options = SqliteConnectOptions::from_str(DB_NAME)
        .unwrap()
        .log_statements(tracing::log::LevelFilter::Trace)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(sqlite_options)
        .await?;

    init_db(&pool).await?;

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_origin(tower_http::cors::Any)
        .allow_headers([CONTENT_TYPE]);

    let assets_path = std::env::current_dir().unwrap();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route(
            "/todos",
            post(controllers::todo::create_todo_from_form).get(controllers::todo::get_todos_list),
        )
        .route("/todos/:id", delete(controllers::todo::delete_todo))
        .route(
            "/todos/:id/toggle",
            post(controllers::todo::toggle_todo_state),
        )
        .route(
            "/todos/filter/:filter",
            get(controllers::todo::get_filtered_todos),
        )
        .route("/login", get(login))
        // .route("/assets/styles.css", get(style))
        .nest(
            controllers::task::html_api::NEST_PREFIX,
            controllers::task::html_api::router(pool.clone()),
        )
        .with_state(pool.clone())
        .nest("/api", build_api_router(pool))
        // workspace has the packages nested from the root, so we need to specify 'appserver/assets'
        .nest_service("/assets", ServeDir::new(config.assets_dir))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn init_db(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    data::migrate_table_state(pool).await?;

    // let test = data::select_task_row(1, pool).await?;
    // tracing::info!("{:#?}", test);

    // let task_todos = data::query_todos_by_task(test.0, pool).await?;
    // tracing::info!("{:#?}", task_todos);

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn root() -> impl IntoResponse {
    IndexTemplate
}

async fn login() -> impl IntoResponse {
    LoginIndex
}

async fn read_config() -> config::Config {
    let config: config::Config;
    if let Ok(mut file) = tokio::fs::File::open("config.json").await {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).await.unwrap();
        config = serde_json::from_str(&buffer).unwrap();
    } else {
        config = config::Config::default();
    }
    config
}

async fn save_backup(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let config: config::Config;

    if let Ok(mut file) = tokio::fs::File::open("config.json").await {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).await.unwrap();
        config = serde_json::from_str(&buffer).unwrap();
    } else {
        config = config::Config::default();
    }

    let mut file_name = std::path::PathBuf::from(config.backup_dir);

    file_name.push("sqlite://data.db");
    tracing::debug!("Saving backup to {:?}", file_name.to_str());

    // TODO: Does not work for some reason, no error,
    // but also no backup file created.
    // Maybe I need to use a proper db file instead ? :(
    match sqlx::query(
        r#"
            VACUUM main INTO 'sqlite://backup.db';
        "#,
    )
    // .bind(file_name.to_str().unwrap())
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

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here

    match sqlx::query("INSERT INTO users (name) Values (?1);")
        .bind(&payload.username)
        .execute(&pool)
        .await
    {
        Ok(query_result) => {
            let id = query_result.last_insert_rowid();
            (
                StatusCode::CREATED,
                Json(User {
                    id: id.try_into().unwrap(),
                    username: payload.username,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(User {
                id: 0,
                username: e.to_string(),
            }),
        ),
    }
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Default)]
struct User {
    id: u64,
    username: String,
}
