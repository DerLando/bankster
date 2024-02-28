pub(crate) mod task;
pub(crate) mod todo;
use paste::paste;

pub(crate) use todo::{create_todo, delete_todo, get_all_todos, read_todo, update_todo};

macro_rules! crud_create {
    ($name:ident, $result:ty, [$($field_name:ident : $field_type:ty)*]) => {
        paste! {
        #[derive(serde::Deserialize)]
        pub(crate) struct [<Create $name>] {
        }

        pub(crate) async fn [<create_ $name>](
                axum::extract::State(pool): axum::extract::State<sqlx::SqlitePool>,
                axum::Json(payload): axum::Json<[<Create $name>]>,)
            -> (axum::http::StatusCode, axum::Json<$result>) {
                // $create_fn()
                todo!()
            }

        }
    };
}

// struct HelloResult;
// crud_create!(Hello, HelloResult, [name: String], (|| {
// todo!()
// }));

// struct TagResult;
// crud_create!(Tag, TagResult, [name: String color: String]);
