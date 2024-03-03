use axum::routing;
use axum::Router;

pub const NEST_PREFIX: &str = "/todos";

pub fn router(state: &sqlx::SqlitePool) -> Router<sqlx::SqlitePool> {
    Router::new()
        .with_state(state.clone())
        .route("/", routing::get(self::get::index).post(self::post::create))
        .route("/find", routing::get(self::get::by_query))
        .route("/all", routing::get(self::get::all))
        .route(
            "/:id",
            routing::get(self::get::all).delete(self::delete::delete),
        )
        .route("/:id/toggle", routing::put(self::put::toggle_state))
}

mod get {
    use askama_axum::IntoResponse;
    use axum::extract::{Path, Query, State};

    use crate::{
        api::types::todo::TodoQuery,
        error::ApiError,
        models,
        viewmodels::{self, todos::TodoIndex},
    };

    pub async fn index() -> impl IntoResponse {
        TodoIndex {
            filter: "all".to_string(),
        }
    }

    pub async fn all(State(pool): State<sqlx::SqlitePool>) -> Result<impl IntoResponse, ApiError> {
        let todos = models::todo::get_all(&pool).await?;
        Ok(viewmodels::todos::TodoListModel {
            filter: "all".to_string(),
            todos,
        })
    }

    pub async fn by_query(
        Query(query): Query<TodoQuery>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let todos = models::todo::get_all_matching(&pool, &query).await?;
        let filter = match query.done {
            Some(true) => "completed",
            Some(false) => "active",
            _ => "all",
        };

        Ok(viewmodels::todos::TodoListModel {
            filter: filter.to_string(),
            todos,
        })
    }

    pub async fn by_index(
        Path(id): Path<i64>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let todo = models::todo::get_by_id(&pool, id).await?;
        Ok(viewmodels::todos::TodoModel { todo })
    }
}

mod post {
    use askama_axum::IntoResponse;
    use axum::{extract::State, http::StatusCode, Form};

    use crate::{api::types::todo::CreateTodo, error::ApiError, models, viewmodels};

    pub async fn create(
        State(pool): State<sqlx::SqlitePool>,
        Form(payload): Form<CreateTodo>,
    ) -> Result<impl IntoResponse, ApiError> {
        let id = models::todo::create(&pool, &payload).await?;
        let todo = models::todo::get_by_id(&pool, id).await?;
        Ok(viewmodels::todos::TodoModel { todo })
    }
}

mod put {
    use askama_axum::IntoResponse;
    use axum::extract::{Path, State};

    use crate::{error::ApiError, models, viewmodels};

    pub async fn toggle_state(
        Path(id): Path<i64>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let _ = models::todo::toggle_state(&pool, id).await?;
        let todo = models::todo::get_by_id(&pool, id).await?;
        Ok(viewmodels::todos::TodoModel { todo })
    }
}

mod delete {
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
    };

    use crate::{error::ApiError, models};

    pub async fn delete(
        Path(id): Path<i64>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        models::todo::delete(&pool, id)
            .await
            .map(|_| StatusCode::OK)
            .map_err(|e| e.into())
    }
}
