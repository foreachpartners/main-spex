use super::*;

fn state() -> AppState {
    AppState {
        menu: Arc::new(menu()),
        deliveries: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(AtomicI64::new(1)),
    }
}

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn order(pizza_ids: Vec<i64>) -> NewOrder {
    NewOrder { pizza_ids }
}

#[tokio::test]
async fn list_pizzas_returns_the_five_pizza_menu() {
    let menu = list_pizzas(State(state())).await.0;
    assert_eq!(menu.len(), 5);
    assert_eq!(menu[0].name, "Margherita");
}

#[tokio::test]
async fn deliveries_start_empty() {
    let deliveries = list_deliveries(State(state())).await.0;
    assert!(deliveries.is_empty());
}

#[tokio::test]
async fn create_order_starts_a_delivery_with_pizza_names() {
    let state = state();

    let response = create_order(State(state.clone()), Json(order(vec![1, 2]))).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["id"], 1);
    assert_eq!(body["items"], json!(["Margherita", "Pepperoni"]));

    let deliveries = list_deliveries(State(state)).await.0;
    assert_eq!(deliveries.len(), 1);
}

#[tokio::test]
async fn create_order_rejects_an_empty_cart() {
    let state = state();

    let response = create_order(State(state.clone()), Json(order(vec![]))).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(list_deliveries(State(state)).await.0.is_empty());
}

#[tokio::test]
async fn create_order_rejects_unknown_pizza() {
    let state = state();

    let response = create_order(State(state.clone()), Json(order(vec![99]))).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(list_deliveries(State(state)).await.0.is_empty());
}

#[tokio::test]
async fn list_deliveries_drops_expired_ones() {
    let state = state();
    create_order(State(state.clone()), Json(order(vec![1]))).await;

    let expired = Instant::now().checked_sub(DELIVERY_TTL).unwrap();
    state.deliveries.lock().unwrap()[0].created_at = expired;

    assert!(list_deliveries(State(state)).await.0.is_empty());
}
