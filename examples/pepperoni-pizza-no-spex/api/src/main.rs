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

// A delivery disappears from the API this long after it is created.
const DELIVERY_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Serialize)]
struct Pizza {
    id: i64,
    name: String,
    price: f64,
}

#[derive(Clone, Serialize)]
struct Delivery {
    id: i64,
    items: Vec<String>,
    #[serde(skip)]
    created_at: Instant,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewOrder {
    pizza_ids: Vec<i64>,
}

#[derive(Clone)]
struct AppState {
    menu: Arc<Vec<Pizza>>,
    deliveries: Arc<Mutex<Vec<Delivery>>>,
    next_id: Arc<AtomicI64>,
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
    let state = AppState {
        menu: Arc::new(menu()),
        deliveries: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(AtomicI64::new(1)),
    };

    let app = Router::new()
        .route("/pizzas", get(list_pizzas))
        .route("/orders", post(create_order))
        .route("/deliveries", get(list_deliveries))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");

    println!("Pepperoni Pizza API listening on http://127.0.0.1:3000");
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

    let delivery = Delivery {
        id: state.next_id.fetch_add(1, Ordering::Relaxed),
        items,
        created_at: Instant::now(),
    };

    state.deliveries.lock().unwrap().push(delivery.clone());
    (StatusCode::CREATED, Json(delivery)).into_response()
}

async fn list_deliveries(State(state): State<AppState>) -> Json<Vec<Delivery>> {
    let mut deliveries = state.deliveries.lock().unwrap();
    deliveries.retain(|delivery| delivery.created_at.elapsed() < DELIVERY_TTL);
    Json(deliveries.clone())
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}

#[cfg(test)]
mod tests;
