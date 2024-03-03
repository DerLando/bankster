use askama::Template;
use piapi_rs::types::{task::TaskHeader, todo::TodoItem};
#[derive(Template)]
#[template(path = "task_index.html")]
pub(crate) struct TaskIndex;

#[derive(Template)]
#[template(path = "tasks.html")]
pub(crate) struct Tasks {
    pub tasks: Vec<crate::controllers::task::TaskHeader>,
}

#[derive(Template)]
#[template(path = "tasks/edit.html")]
pub(crate) struct TaskEdit {
    pub task: crate::controllers::task::TaskModel,
}
