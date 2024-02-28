use askama::Template;
use piapi_rs::types::{task::TaskHeader, todo::TodoItem};

// #[derive(Template)]
// #[template(path = "todos.html")]
// pub(crate) struct TaskHeadersTemplate {
//     tasks: Vec<TaskHeaderTemplate>,
// }

// #[derive(Template)]
// pub(crate) struct TaskHeaderTemplate {
//     id: u64,
//     name: String,
// }

#[derive(Template)]
#[template(path = "todo.html")]
pub(crate) struct TodoModel {
    pub todo: TodoItem,
}

#[derive(Template)]
#[template(path = "todos.html")]
pub(crate) struct TodoListModel {
    pub filter: String,
    pub todos: Vec<TodoItem>,
}

#[derive(Template)]
#[template(path = "todo_index.html")]
pub(crate) struct TodoIndex {
    pub filter: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub(crate) struct LoginIndex;

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
