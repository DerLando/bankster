pub mod task;
pub mod todo;
pub mod utils {
    use chrono::Utc;

    pub(crate) type DateTime = sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>;
    pub(crate) fn now() -> DateTime {
        Utc::now()
    }
}
