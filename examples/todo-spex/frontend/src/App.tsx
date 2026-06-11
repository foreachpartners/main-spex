import { useEffect, useState } from "react";
import { todosApi } from "./api";
import type { Todo } from "./types";

export default function App() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [title, setTitle] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    todosApi.list().then(setTodos).catch(showError);
  }, []);

  function showError(err: unknown) {
    setError(err instanceof Error ? err.message : "Something went wrong");
  }

  async function addTodo(event: React.FormEvent) {
    event.preventDefault();
    const trimmed = title.trim();
    if (!trimmed) return;

    try {
      const created = await todosApi.create({ title: trimmed });
      setTodos((current) => [...current, created]);
      setTitle("");
    } catch (err) {
      showError(err);
    }
  }

  async function toggleTodo(todo: Todo) {
    try {
      const updated = await todosApi.update(todo.id, {
        completed: !todo.completed,
      });
      setTodos((current) =>
        current.map((item) => (item.id === updated.id ? updated : item)),
      );
    } catch (err) {
      showError(err);
    }
  }

  async function removeTodo(id: number) {
    try {
      await todosApi.remove(id);
      setTodos((current) => current.filter((item) => item.id !== id));
    } catch (err) {
      showError(err);
    }
  }

  return (
    <main className="app">
      <h1>Todo</h1>

      <form className="add-form" onSubmit={addTodo}>
        <input
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="What needs to be done?"
          aria-label="New todo title"
        />
        <button type="submit">Add</button>
      </form>

      {error && <p className="error">{error}</p>}

      <ul className="todo-list">
        {todos.map((todo) => (
          <li key={todo.id} className={todo.completed ? "done" : ""}>
            <label>
              <input
                type="checkbox"
                checked={todo.completed}
                onChange={() => toggleTodo(todo)}
              />
              <span>{todo.title}</span>
            </label>
            <button
              className="remove"
              onClick={() => removeTodo(todo.id)}
              aria-label={`Delete ${todo.title}`}
            >
              ✕
            </button>
          </li>
        ))}
      </ul>

      {todos.length === 0 && !error && <p className="empty">No todos yet.</p>}
    </main>
  );
}
