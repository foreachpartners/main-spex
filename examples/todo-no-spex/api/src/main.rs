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
struct Todo {
    id: i64,
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
    #[serde(default)]
    completed: bool,
}

#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

#[derive(Clone, Default)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    next_id: Arc<AtomicI64>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(AtomicI64::new(1)),
    };

    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/:id",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");

    println!("Todo API listening on http://127.0.0.1:3000");
    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn list_todos(State(state): State<AppState>) -> Json<Vec<Todo>> {
    let todos = state.todos.lock().unwrap();
    Json(todos.clone())
}

async fn create_todo(
    State(state): State<AppState>,
    Json(input): Json<NewTodo>,
) -> Response {
    if input.title.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "title must not be empty");
    }

    let todo = Todo {
        id: state.next_id.fetch_add(1, Ordering::Relaxed),
        title: input.title,
        completed: input.completed,
    };

    state.todos.lock().unwrap().push(todo.clone());
    (StatusCode::CREATED, Json(todo)).into_response()
}

async fn get_todo(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let todos = state.todos.lock().unwrap();
    match todos.iter().find(|todo| todo.id == id) {
        Some(todo) => Json(todo.clone()).into_response(),
        None => not_found(),
    }
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTodo>,
) -> Response {
    let mut todos = state.todos.lock().unwrap();
    let Some(todo) = todos.iter_mut().find(|todo| todo.id == id) else {
        return not_found();
    };

    if let Some(title) = input.title {
        todo.title = title;
    }
    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    Json(todo.clone()).into_response()
}

async fn delete_todo(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let mut todos = state.todos.lock().unwrap();
    let len_before = todos.len();
    todos.retain(|todo| todo.id != id);

    if todos.len() == len_before {
        not_found()
    } else {
        StatusCode::NO_CONTENT.into_response()
    }
}

fn not_found() -> Response {
    error(StatusCode::NOT_FOUND, "todo not found")
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}
