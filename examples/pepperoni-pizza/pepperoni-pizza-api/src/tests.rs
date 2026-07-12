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

// @spec test FS-DELIVERY-TIME
#[tokio::test]
async fn estimate_rounds_up_over_two_workers() {
    let response = estimate_order_delivery_time(
        State(state()),
        Json(EstimateRequest {
            pizza_ids: vec![1, 2, 3],
        }),
    )
    .await;

    let body = body_json(response).await;
    assert_eq!(body["estimatedSeconds"], 120);
}

#[tokio::test]
async fn estimate_counts_in_progress_workload() {
    let state = state();
    create_order(State(state.clone()), Json(order(vec![1, 2]))).await;

    let response = estimate_order_delivery_time(
        State(state),
        Json(EstimateRequest {
            pizza_ids: vec![1],
        }),
    )
    .await;

    let body = body_json(response).await;
    assert_eq!(body["estimatedSeconds"], 120);
}

#[tokio::test]
async fn order_estimate_validates_pizza_ids() {
    let valid = estimate_order_delivery_time(
        State(state()),
        Json(EstimateRequest {
            pizza_ids: vec![1],
        }),
    )
    .await;
    assert_eq!(valid.status(), StatusCode::OK);
    let body = body_json(valid).await;
    assert!(body["estimatedSeconds"].is_number());

    let unknown = estimate_order_delivery_time(
        State(state()),
        Json(EstimateRequest {
            pizza_ids: vec![99],
        }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
    let body = body_json(unknown).await;
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn deliveries_carry_the_estimate() {
    let state = state();

    let response = create_order(State(state.clone()), Json(order(vec![1, 2, 3]))).await;
    let body = body_json(response).await;
    assert_eq!(body["estimatedSeconds"], 120);

    let deliveries = list_deliveries(State(state)).await.0;
    assert_eq!(deliveries[0].estimated_seconds, 120);
}

#[tokio::test]
async fn list_deliveries_drops_orders_past_their_estimate() {
    let state = state();
    create_order(State(state.clone()), Json(order(vec![1]))).await;

    // Age the delivery past its estimated delivery time so it counts as arrived.
    {
        let mut deliveries = state.deliveries.lock().unwrap();
        let window = deliveries[0].delivery_window();
        deliveries[0].created_at = Instant::now()
            .checked_sub(window + Duration::from_secs(1))
            .unwrap();
    }

    assert!(list_deliveries(State(state)).await.0.is_empty());
}

#[tokio::test]
async fn list_deliveries_keeps_orders_within_their_estimate() {
    let state = state();
    create_order(State(state.clone()), Json(order(vec![1]))).await;

    // A freshly created delivery is within its estimate and stays in progress.
    assert_eq!(list_deliveries(State(state)).await.0.len(), 1);
}

#[tokio::test]
async fn list_deliveries_counts_down_remaining_time() {
    let state = state();
    create_order(State(state.clone()), Json(order(vec![1, 2, 3]))).await;

    // Age the delivery by 30 seconds of its 120-second estimate.
    state.deliveries.lock().unwrap()[0].created_at =
        Instant::now().checked_sub(Duration::from_secs(30)).unwrap();

    let deliveries = list_deliveries(State(state)).await.0;
    assert_eq!(deliveries[0].estimated_seconds, 120);
    assert_eq!(deliveries[0].remaining_seconds, 90);
}
