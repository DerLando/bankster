use crate::error::ApiError;

#[derive(serde::Deserialize)]
pub struct TodoItems {
    pub items: Vec<TodoItem>,
}

#[derive(
    serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug, Default, sqlx::FromRow,
)]
pub struct TodoItem {
    pub id: i64,
    pub name: String,
    pub done: bool,
}

impl From<(i64, String, bool)> for TodoItem {
    fn from(value: (i64, String, bool)) -> Self {
        Self {
            id: value.0,
            name: value.1,
            done: value.2,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateTodo {
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct UpdateTodo {
    pub name: String,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct TodoQuery {
    pub done: Option<bool>,
}
