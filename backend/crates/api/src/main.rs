mod config;
mod errors;
mod handlers;
mod middleware;
pub mod requests;
pub mod responses;

use std::sync::Arc;

use axum::routing::{get, patch};
use axum::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::AppConfig;
use handlers::AppState;
use middleware::RequestIdGenerator;

use db::repos::{
    SqliteBudgetEntryRepository, SqliteCategoryRepository, SqliteMonthRepository,
    SqliteTransactionRepository,
};
use domain::services::{
    CategoryService, EntryService, MonthService, SummaryService, TransactionService,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::load().expect("Failed to load configuration");

    let pool = db::create_pool(&config.database.url)
        .await
        .expect("Failed to create database pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    // Create repository instances
    let category_repo = Arc::new(SqliteCategoryRepository::new(pool.clone()));
    let month_repo = Arc::new(SqliteMonthRepository::new(pool.clone()));
    let entry_repo = Arc::new(SqliteBudgetEntryRepository::new(pool.clone()));
    let transaction_repo = Arc::new(SqliteTransactionRepository::new(pool.clone()));

    // Create service instances
    let category_service = Arc::new(CategoryService::new(category_repo.clone()));
    let month_service = Arc::new(MonthService::new(
        month_repo.clone(),
        entry_repo.clone(),
    ));
    let entry_service = Arc::new(EntryService::new(
        entry_repo.clone(),
        category_repo.clone(),
        month_repo.clone(),
    ));
    let transaction_service = Arc::new(TransactionService::new(
        transaction_repo.clone(),
        entry_repo.clone(),
    ));
    let summary_service = Arc::new(SummaryService::new(
        entry_repo.clone(),
        transaction_repo.clone(),
        month_repo.clone(),
    ));

    let state = AppState {
        category_service,
        month_service,
        entry_service,
        transaction_service,
        summary_service,
        currency_config: config.currency.clone(),
    };

    // Configure CORS
    let origins: Vec<_> = config
        .cors
        .allowed_origins
        .iter()
        .map(|o| o.parse().expect("Invalid CORS origin"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    // Build router
    let api = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route(
            "/categories",
            get(handlers::categories::list_categories)
                .post(handlers::categories::create_category),
        )
        .route(
            "/categories/{id}",
            patch(handlers::categories::update_category),
        )
        .route(
            "/months",
            get(handlers::months::list_months).post(handlers::months::create_month),
        )
        .route("/months/{id}", get(handlers::months::get_month))
        .route(
            "/months/{id}/entries",
            get(handlers::entries::list_entries).post(handlers::entries::create_entry),
        )
        .route(
            "/months/{id}/entries/{entry_id}",
            patch(handlers::entries::update_entry).delete(handlers::entries::delete_entry),
        )
        .route(
            "/transactions",
            get(handlers::transactions::list_transactions)
                .post(handlers::transactions::create_transaction),
        )
        .route(
            "/transactions/{id}",
            patch(handlers::transactions::update_transaction)
                .delete(handlers::transactions::delete_transaction),
        )
        .route(
            "/months/{id}/summary",
            get(handlers::summary::get_month_summary),
        );

    let app = Router::new()
        .nest("/api/v1", api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(RequestIdGenerator::default()))
        .layer(PropagateRequestIdLayer::x_request_id())
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
