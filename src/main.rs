mod db;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router, Server,
};
use chrono::{DateTime, Utc};
use tokio_nsq::{NSQProducerConfig, NSQProducer, NSQTopic};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tokio::sync::Mutex;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Task {
    id: i32,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateTask {
    title: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateTask {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    nsq_producer: Arc<Mutex<NSQProducer>>,
}

impl AppState {
    pub async fn new(pool: PgPool, nsq_producer: NSQProducer) -> Self {
        Self {
            pool,
            nsq_producer: Arc::new(Mutex::new(nsq_producer)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")?;
    let db = PgPool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    // Initialize NSQ producer
    let nsqd_url = std::env::var("NSQD_URL").unwrap_or_else(|_| "127.0.0.1:4150".to_string());
    let producer = NSQProducerConfig::new(nsqd_url)
        .build();
    let nsq_producer = Arc::new(Mutex::new(producer));

    // Create application state
    let state = AppState { pool: db, nsq_producer };

    // Create router
    let app = Router::new()
        .route("/tasks", get(list_tasks))
        .route("/tasks", post(create_task))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = "127.0.0.1:3000";
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let std_listener = listener.into_std()?;
    Server::from_tcp(std_listener)?.serve(app.into_make_service()).await?;

    Ok(())
}

async fn list_tasks(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match db::list_tasks(&state.pool).await {
        Ok(tasks) => (StatusCode::OK, axum::Json(tasks)),
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(Vec::<Task>::new()))
        }
    }
}

async fn create_task(
    State(state): State<AppState>,
    axum::Json(task): axum::Json<CreateTask>,
) -> impl IntoResponse {
    match db::create_task(&state.pool, task.title, task.description).await {
        Ok(task) => (StatusCode::CREATED, axum::Json(task)),
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(Task {
                id: 0,
                title: String::new(),
                description: None,
                completed: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }
    }
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match db::get_task(&state.pool, id).await {
        Ok(Some(task)) => (StatusCode::OK, axum::Json(task)),
        Ok(None) => (StatusCode::NOT_FOUND, axum::Json(Task {
            id: 0,
            title: String::new(),
            description: None,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(Task {
                id: 0,
                title: String::new(),
                description: None,
                completed: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }
    }
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    axum::Json(task): axum::Json<UpdateTask>,
) -> Result<axum::Json<Task>, StatusCode> {
    let result = db::update_task(
        &state.pool,
        id,
        task.title,
        task.description,
        task.completed,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Publish update to NSQ
    let message = serde_json::to_vec(&result).unwrap();
    
    if let Err(e) = publish_message(&state.nsq_producer, "task_updates", message).await {
        eprintln!("Failed to publish message: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::Json(result))
}

async fn publish_message(producer: &Mutex<NSQProducer>, topic_str: &str, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut producer = producer.lock().await;
    let topic = Arc::new(NSQTopic::new(topic_str).unwrap());
    producer.publish(&topic, message).await?;
    Ok(())
}
