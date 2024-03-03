// mod tasks;
pub mod todos;

use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub(crate) struct LoginIndex;
