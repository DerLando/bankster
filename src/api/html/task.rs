use askama_axum::IntoResponse;
use axum::{routing, Router};

pub(crate) const NEST_PREFIX: &str = "/tasks";

pub fn router(state: sqlx::SqlitePool) -> Router<sqlx::SqlitePool> {
    Router::new()
        .with_state(state)
        .route("/", routing::get(self::get::index))
        .route("/all", routing::get(self::get::headers))
        .route("/:id/edit", routing::get(self::get::edit))
        .route("/:id/details", routing::get(self::get::details))
}

mod get {
    use askama_axum::IntoResponse;
    use axum::extract::{Path, Query, State};

    use crate::{error::ApiError, models, viewmodels};

    pub(super) async fn index() -> impl IntoResponse {
        viewmodels::tasks::TaskIndex
    }

    pub(super) async fn headers(
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let headers = models::task::get_all_headers(&pool).await?;
        Ok(viewmodels::tasks::Tasks { tasks: headers })
    }

    pub(super) async fn edit(
        Path(id): Path<i64>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let task = models::task::get_by_id(&pool, id).await?;

        Ok(viewmodels::tasks::TaskEdit { task })
    }

    pub(super) async fn details(
        Path(id): Path<i64>,
        State(pool): State<sqlx::SqlitePool>,
    ) -> Result<impl IntoResponse, ApiError> {
        let task = models::task::get_by_id(&pool, id).await?;

        Ok(viewmodels::tasks::TaskDetails { task })
    }
}

mod post {
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, Query, State},
        Form,
    };

    use crate::{api::types::task::CreateTask, error::ApiError, models, viewmodels};

    pub(super) async fn create(
        State(pool): State<sqlx::SqlitePool>,
        Form(payload): Form<CreateTask>,
    ) -> Result<impl IntoResponse, ApiError> {
        let id = models::task::create_task(&pool, &payload).await?;
        let task = models::task::get_by_id(&pool, id).await?;

        // TODO: There is no proper viewmodel to return here yet...
        Ok(viewmodels::tasks::TaskIndex)
    }
}
