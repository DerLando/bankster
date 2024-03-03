use sqlx::types::chrono::{TimeZone, Utc};
use sqlx::Row;

// TODO: data module is kind of awkward, as it's functioniality is repeated somewhat in the models module. Maybe this should be data_access, and models use methods in here?

pub(crate) trait TableRow {
    type Data;
    type Update;

    async fn select(id: i64, pool: &sqlx::SqlitePool) -> Result<Self::Data, sqlx::Error>;
    async fn insert(update: Self::Update, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error>;
    async fn update(
        id: i64,
        update: Self::Update,
        pool: &sqlx::SqlitePool,
    ) -> Result<(), sqlx::Error>;
}

pub(crate) struct TodoRow;
impl TableRow for TodoRow {
    type Data = (i64, String, bool);

    type Update = (String, bool);

    async fn select(id: i64, pool: &sqlx::SqlitePool) -> Result<Self::Data, sqlx::Error> {
        sqlx::query(
            r#"
            SELECT * FROM todos
            WHERE id = (?1)
        ;
        "#,
        )
        .bind(id)
        .map(|row: sqlx::sqlite::SqliteRow| (row.get("id"), row.get("name"), row.get("done")))
        .fetch_one(pool)
        .await
    }

    async fn update(
        id: i64,
        update: Self::Update,
        pool: &sqlx::SqlitePool,
    ) -> Result<(), sqlx::Error> {
        todo!()
    }

    async fn insert(data: Self::Update, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        let (name, done) = data;
        sqlx::query("INSERT INTO todos (name, done) Values (?1, ?2);")
            .bind(&name)
            .bind(&done)
            .execute(pool)
            .await
            .map(|result| result.last_insert_rowid())
    }
}

/// (name)
pub(crate) type TagRow = (i64, String, String);
/// (index, name, description, created, due, done)
pub(crate) type TaskRow = (i64, String, String, DateTime, DateTime, bool);
pub(crate) type DateTime = sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>;
pub(crate) fn now() -> DateTime {
    Utc::now()
}

pub(crate) async fn create_todo_table(pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos
        (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT,
            done BOOLEAN
        );
        "#,
    )
    .execute(pool)
    .await;
}

pub(crate) async fn create_tasks_table(pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks
        (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT,
            description TEXT,
            created TEXT,
            due TEXT,
            done BOOLEAN
        );
        "#,
    )
    .execute(pool)
    .await;
}

pub(crate) async fn create_tags_table(pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags
        (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT,
            color TEXT
        );
        "#,
    )
    .execute(pool)
    .await;
}

pub(crate) async fn create_task_todos_mapping_table(pool: &sqlx::SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasktodos
        (
            task_id INTEGER NOT NULL,
            todo_id INTEGER NOT NULL,
            foreign key (task_id) references tasks(id),
            foreign key (todo_id) references todos(id),
            primary key (task_id, todo_id)
        );
        "#,
    )
    .execute(pool)
    .await;
}

pub(crate) async fn select_todo_row(
    id: i64,
    pool: &sqlx::SqlitePool,
) -> Option<<TodoRow as TableRow>::Data> {
    TodoRow::select(id, pool).await.ok()
}

pub(crate) async fn insert_todo_row(
    todo: <TodoRow as TableRow>::Update,
    pool: &sqlx::SqlitePool,
) -> Result<i64, sqlx::Error> {
    TodoRow::insert(todo, pool).await
}

pub(crate) async fn select_task_row(
    id: i64,
    pool: &sqlx::SqlitePool,
) -> Result<TaskRow, sqlx::Error> {
    sqlx::query(
        r#"
            SELECT * FROM tasks
            WHERE id = (?1)
        ;
        "#,
    )
    .bind(id)
    .map(|row: sqlx::sqlite::SqliteRow| {
        (
            row.get("id"),
            row.get("name"),
            row.get("description"),
            row.get("created"),
            row.get("due"),
            row.get("done"),
        )
    })
    .fetch_one(pool)
    .await
}

pub(crate) type TaskRowInput = (String, String, DateTime, bool);
pub(crate) async fn insert_task_row(
    input: TaskRowInput,
    pool: &sqlx::SqlitePool,
) -> Result<i64, sqlx::Error> {
    let (name, description, due, done) = input;
    sqlx::query(
        r#"
          INSERT INTO tasks (name, description, created, due, done)
        VALUES (?1, ?2, ?3, ?4, ?5);  
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(Utc::now())
    .bind(due)
    .bind(done)
    .execute(pool)
    .await
    .map(|result| result.last_insert_rowid())
}

pub(crate) async fn query_todos_by_task(
    task_id: i64,
    pool: &sqlx::SqlitePool,
) -> Result<Vec<<TodoRow as TableRow>::Data>, sqlx::Error> {
    sqlx::query(
        r#"
          SELECT * FROM todos 
          JOIN tasktodos tt ON tt.task_id = (?1) AND tt.todo_id = id
        ;
        "#,
    )
    .bind(task_id)
    .map(|row: sqlx::sqlite::SqliteRow| (row.get("id"), row.get("name"), row.get("done")))
    .fetch_all(pool)
    .await
}

pub(crate) async fn migrate_table_state(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY NOT NULL, name TEXT);",
    )
    .execute(pool)
    .await?;

    create_todo_table(pool).await;
    create_tasks_table(pool).await;
    create_tags_table(pool).await;
    create_task_todos_mapping_table(pool).await;
    // create_mock_data(pool).await?;

    Ok(())
}

pub(crate) async fn create_mock_data(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    insert_todo_row(("Kiss Jana".to_string(), false), pool).await?;
    insert_todo_row(
        ("Add One to many from tasks to todos".to_string(), false),
        pool,
    )
    .await?;
    insert_todo_row(
        ("Add one to many from taks to tags".to_string(), false),
        pool,
    )
    .await?;

    insert_task_row(
        (
            "Implement all tables".to_string(),
            "Need to add tables for all data rows, as well as mapping tables".to_string(),
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            false,
        ),
        pool,
    )
    .await?;

    let mapping_query = r#"
            INSERT INTO tasktodos (task_id, todo_id)
            VALUES(?1, ?2)
        "#;

    sqlx::query(&mapping_query)
        .bind(1)
        .bind(1)
        .execute(pool)
        .await?;
    sqlx::query(&mapping_query)
        .bind(1)
        .bind(2)
        .execute(pool)
        .await?;

    Ok(())
}
