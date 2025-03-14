use sqlx::PgPool;
use crate::Task;

pub async fn list_tasks(pool: &PgPool) -> Result<Vec<Task>, sqlx::Error> {
    sqlx::query_as!(
        Task,
        r#"
        SELECT id, title, description, completed, created_at, updated_at
        FROM tasks
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn create_task(pool: &PgPool, title: String, description: Option<String>) -> Result<Task, sqlx::Error> {
    sqlx::query_as!(
        Task,
        r#"
        INSERT INTO tasks (title, description)
        VALUES ($1, $2)
        RETURNING id, title, description, completed, created_at, updated_at
        "#,
        title,
        description
    )
    .fetch_one(pool)
    .await
}

pub async fn get_task(pool: &PgPool, id: i32) -> Result<Option<Task>, sqlx::Error> {
    sqlx::query_as!(
        Task,
        r#"
        SELECT id, title, description, completed, created_at, updated_at
        FROM tasks
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_task(
    pool: &PgPool,
    id: i32,
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
) -> Result<Task, sqlx::Error> {
    sqlx::query_as!(
        Task,
        r#"
        UPDATE tasks
        SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            completed = COALESCE($3, completed),
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, title, description, completed, created_at, updated_at
        "#,
        title,
        description,
        completed,
        id
    )
    .fetch_one(pool)
    .await
} 