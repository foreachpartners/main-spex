use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::CorsLayer;

#[derive(Clone, Serialize)]
struct Order {
    id: i64,
    customer: String,
    delivered: bool,
}

#[derive(Deserialize)]
struct NewOrder {
    customer: String,
    #[serde(default)]
    delivered: bool,
}

#[derive(Deserialize)]
struct UpdateOrder {
    customer: Option<String>,
    delivered: Option<bool>,
}

#[derive(Clone, Default)]
struct AppState {
    orders: Arc<Mutex<Vec<Order>>>,
    next_id: Arc<AtomicI64>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        orders: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(AtomicI64::new(1)),
    };

    let app = Router::new()
        .route("/orders", get(list_orders).post(create_order))
        .route(
            "/orders/{id}",
            get(get_order).put(update_order).delete(delete_order),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");

    println!("Pepperoni Pizza API listening on http://127.0.0.1:3000");
    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn list_orders(State(state): State<AppState>) -> Json<Vec<Order>> {
    let orders = state.orders.lock().unwrap();
    Json(orders.clone())
}

async fn create_order(
    State(state): State<AppState>,
    Json(input): Json<NewOrder>,
) -> Response {
    if input.customer.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "customer must not be empty");
    }

    let order = Order {
        id: state.next_id.fetch_add(1, Ordering::Relaxed),
        customer: input.customer,
        delivered: input.delivered,
    };

    state.orders.lock().unwrap().push(order.clone());
    (StatusCode::CREATED, Json(order)).into_response()
}

async fn get_order(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let orders = state.orders.lock().unwrap();
    match orders.iter().find(|order| order.id == id) {
        Some(order) => Json(order.clone()).into_response(),
        None => not_found(),
    }
}

async fn update_order(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateOrder>,
) -> Response {
    let mut orders = state.orders.lock().unwrap();
    let Some(order) = orders.iter_mut().find(|order| order.id == id) else {
        return not_found();
    };

    if let Some(customer) = input.customer {
        order.customer = customer;
    }
    if let Some(delivered) = input.delivered {
        order.delivered = delivered;
    }

    Json(order.clone()).into_response()
}

async fn delete_order(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let mut orders = state.orders.lock().unwrap();
    let len_before = orders.len();
    orders.retain(|order| order.id != id);

    if orders.len() == len_before {
        not_found()
    } else {
        StatusCode::NO_CONTENT.into_response()
    }
}

fn not_found() -> Response {
    error(StatusCode::NOT_FOUND, "order not found")
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}
