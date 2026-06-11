import type { NewTodo, Todo, UpdateTodo } from "./types";

const BASE_URL = "http://localhost:3000";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });

  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(body?.message ?? `Request failed: ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

export const todosApi = {
  list: () => request<Todo[]>("/todos"),

  create: (todo: NewTodo) =>
    request<Todo>("/todos", {
      method: "POST",
      body: JSON.stringify(todo),
    }),

  update: (id: number, changes: UpdateTodo) =>
    request<Todo>(`/todos/${id}`, {
      method: "PUT",
      body: JSON.stringify(changes),
    }),

  remove: (id: number) =>
    request<void>(`/todos/${id}`, {
      method: "DELETE",
    }),
};
