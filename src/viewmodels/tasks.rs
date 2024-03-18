use crate::api::types::task::{TaskHeader, TaskModel};
use askama::Template;

#[derive(Template)]
#[template(path = "tasks/index.html")]
pub(crate) struct TaskIndex;

#[derive(Template)]
#[template(path = "tasks/tasks.html")]
pub(crate) struct Tasks {
    pub tasks: Vec<TaskHeader>,
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
pub(crate) struct TaskEdit {
    pub task: TaskModel,
}

#[derive(Template)]
#[template(path = "tasks/details.html")]
pub(crate) struct TaskDetails {
    pub task: TaskModel,
}
