use crate::api::types::todo::TodoItem;
use askama::Template;

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
