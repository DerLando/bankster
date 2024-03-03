#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskHeaders {
    pub tasks: Vec<TaskHeader>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskHeader {
    pub id: i64,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskModel {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created: super::time::DateTime,
    pub due: super::time::DateTime,
    pub done: bool,
    pub todos: Vec<super::todo::TodoItem>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateTask {
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateTask {
    pub name: Option<String>,
    pub description: Option<String>,
    pub due: Option<super::time::DateTime>,
    pub done: Option<bool>,
}
