use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::CorsLayer;

// Workers on shift, each making one pizza at a time.
const WORKERS: i64 = 2;
// Minutes to make one pizza.
const MINUTES_PER_PIZZA: i64 = 1;

// Runtime configuration, read from the PEPPERONI_PIZZA_API__ environment.
#[derive(Debug, Deserialize)]
struct Config {
    bind_addr: String,
}

#[derive(Clone, Serialize)]
struct Pizza {
    id: i64,
    name: String,
    price: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Delivery {
    id: i64,
    items: Vec<String>,
    estimated_seconds: i64,
    remaining_seconds: i64,
    #[serde(skip)]
    created_at: Instant,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewOrder {
    pizza_ids: Vec<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EstimateRequest {
    pizza_ids: Vec<i64>,
}

#[derive(Clone)]
struct AppState {
    menu: Arc<Vec<Pizza>>,
    deliveries: Arc<Mutex<Vec<Delivery>>>,
    next_id: Arc<AtomicI64>,
}

// Delivery-time estimate in whole seconds: the workers share the pizzas at
// MINUTES_PER_PIZZA each, rounded up to the next whole minute, then returned in
// seconds.
fn estimate_seconds(order_pizzas: i64, in_progress_pizzas: i64) -> i64 {
    let total = order_pizzas + in_progress_pizzas;
    let minutes = (total * MINUTES_PER_PIZZA + WORKERS - 1) / WORKERS;
    minutes * 60
}

impl Delivery {
    // A delivery stays in progress until its estimated delivery time elapses,
    // then counts as arrived and drops from the API.
    fn delivery_window(&self) -> Duration {
        Duration::from_secs(self.estimated_seconds.max(0) as u64)
    }

    // Whole seconds left until this delivery arrives, counting down from the
    // estimate as time passes, never negative.
    fn remaining_seconds(&self) -> i64 {
        (self.estimated_seconds - self.created_at.elapsed().as_secs() as i64).max(0)
    }
}

fn menu() -> Vec<Pizza> {
    [
        (1, "Margherita", 8.5),
        (2, "Pepperoni", 9.5),
        (3, "Hawaiian", 10.0),
        (4, "Veggie", 9.0),
        (5, "Four Cheese", 11.0),
    ]
    .into_iter()
    .map(|(id, name, price)| Pizza {
        id,
        name: name.to_string(),
        price,
    })
    .collect()
}

#[tokio::main]
async fn main() {
    // Local overrides come from .env; the committed .env.dev supplies defaults
    // for any keys .env omits. Real process env vars take precedence over both.
    dotenvy::from_filename(".env").ok();
    dotenvy::from_filename(".env.dev").ok();

    let config: Config = config::Config::builder()
        .add_source(
            config::Environment::with_prefix("PEPPERONI_PIZZA_API")
                .prefix_separator("__")
                .separator("__"),
        )
        .build()
        .expect("failed to load configuration")
        .try_deserialize()
        .expect("invalid configuration: PEPPERONI_PIZZA_API__BIND_ADDR must be set");

    let state = AppState {
        menu: Arc::new(menu()),
        deliveries: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(AtomicI64::new(1)),
    };

    let app = Router::new()
        .route("/pizzas", get(list_pizzas))
        .route("/orders", post(create_order))
        .route("/order-estimate", post(estimate_order_delivery_time))
        .route("/deliveries", get(list_deliveries))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {}: {e}", config.bind_addr));

    println!("Pepperoni Pizza API listening on http://{}", config.bind_addr);
    axum::serve(listener, app).await.expect("server error");
}

async fn list_pizzas(State(state): State<AppState>) -> Json<Vec<Pizza>> {
    Json(state.menu.as_ref().clone())
}

async fn create_order(State(state): State<AppState>, Json(input): Json<NewOrder>) -> Response {
    if input.pizza_ids.is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "order must contain at least one pizza",
        );
    }

    let mut items = Vec::new();
    for id in &input.pizza_ids {
        match state.menu.iter().find(|pizza| pizza.id == *id) {
            Some(pizza) => items.push(pizza.name.clone()),
            None => return error(StatusCode::BAD_REQUEST, "unknown pizza in order"),
        }
    }

    let mut deliveries = state.deliveries.lock().unwrap();
    let in_progress_pizzas: i64 = deliveries
        .iter()
        .map(|delivery| delivery.items.len() as i64)
        .sum();

    let estimated_seconds = estimate_seconds(items.len() as i64, in_progress_pizzas);
    let delivery = Delivery {
        id: state.next_id.fetch_add(1, Ordering::Relaxed),
        estimated_seconds,
        remaining_seconds: estimated_seconds,
        items,
        created_at: Instant::now(),
    };

    deliveries.push(delivery.clone());
    drop(deliveries);
    (StatusCode::CREATED, Json(delivery)).into_response()
}

// @spec impl DELIVERY-TIME
async fn estimate_order_delivery_time(
    State(state): State<AppState>,
    Json(input): Json<EstimateRequest>,
) -> Response {
    for id in &input.pizza_ids {
        if !state.menu.iter().any(|pizza| pizza.id == *id) {
            return error(StatusCode::BAD_REQUEST, "unknown pizza in order");
        }
    }

    let in_progress_pizzas: i64 = state
        .deliveries
        .lock()
        .unwrap()
        .iter()
        .map(|delivery| delivery.items.len() as i64)
        .sum();

    let seconds = estimate_seconds(input.pizza_ids.len() as i64, in_progress_pizzas);
    Json(json!({ "estimatedSeconds": seconds })).into_response()
}

async fn list_deliveries(State(state): State<AppState>) -> Json<Vec<Delivery>> {
    let mut deliveries = state.deliveries.lock().unwrap();
    deliveries.retain(|delivery| delivery.created_at.elapsed() < delivery.delivery_window());
    let current = deliveries
        .iter()
        .map(|delivery| Delivery {
            remaining_seconds: delivery.remaining_seconds(),
            ..delivery.clone()
        })
        .collect();
    Json(current)
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}

#[cfg(test)]
mod tests;
