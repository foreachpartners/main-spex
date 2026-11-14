import { useEffect, useState, type FormEvent } from "react";

interface Item {
  id: string;
  name: string;
  bought: boolean;
}

const API_BASE_URL = "http://localhost:3000";

export default function Home() {
  const [items, setItems] = useState<Item[]>([]);
  const [newItemName, setNewItemName] = useState("");

  useEffect(() => {
    // @spec impl fe-view-shopping-list#load-list-from-api
    fetch(`${API_BASE_URL}/items`)
      .then((response) => response.json())
      .then((data: { items: Item[] }) => setItems(data.items));
  }, []);

  async function handleAddItem(event: FormEvent) {
    event.preventDefault();

    // @spec impl fe-add-shopping-item#reject-empty-submit
    if (newItemName.trim().length === 0) {
      return;
    }

    // @spec impl fe-add-shopping-item#call-create-item-endpoint
    const response = await fetch(`${API_BASE_URL}/items`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: newItemName }),
    });
    const item = (await response.json()) as Item;

    setItems((existing) => [...existing, item]);
    // @spec impl fe-add-shopping-item#clear-input-after-add
    setNewItemName("");
  }

  async function handleToggleBought(item: Item) {
    // @spec impl fe-mark-item-bought#call-mark-bought-endpoint
    const response = await fetch(`${API_BASE_URL}/items/${item.id}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ bought: !item.bought }),
    });
    const updated = (await response.json()) as Item;

    setItems((existing) =>
      existing.map((existingItem) =>
        existingItem.id === updated.id ? updated : existingItem,
      ),
    );
  }

  async function handleDeleteItem(id: string) {
    // @spec impl fe-delete-shopping-item#call-delete-item-endpoint
    await fetch(`${API_BASE_URL}/items/${id}`, { method: "DELETE" });
    // @spec impl fe-delete-shopping-item#remove-item-from-view
    setItems((existing) => existing.filter((item) => item.id !== id));
  }

  async function handleClearAll() {
    // @spec impl fe-clear-shopping-list#confirm-before-clearing
    if (!window.confirm("Clear all items from the shopping list?")) {
      return;
    }

    // @spec impl fe-clear-shopping-list#call-clear-list-endpoint
    await fetch(`${API_BASE_URL}/items`, { method: "DELETE" });
    setItems([]);
  }

  return (
    <div className="shopping-list">
      <p className="shopping-list__title">Shopping notes</p>
      {/* @spec impl fe-add-shopping-item#add-item-input */}
      <form className="shopping-list__add" onSubmit={handleAddItem}>
        <input
          className="shopping-list__input"
          type="text"
          placeholder="Add an item"
          value={newItemName}
          onChange={(event) => setNewItemName(event.target.value)}
        />
        <button className="shopping-list__add-button" type="submit">
          Add
        </button>
      </form>
      {items.length === 0 ? (
        // @spec impl fe-view-shopping-list#empty-state
        <div className="shopping-list__empty">No items yet</div>
      ) : (
        // @spec impl fe-view-shopping-list#display-item-list
        <div className="shopping-list__items">
          {items.map((item) => (
            <div key={item.id} className="shopping-list__item">
              {/* @spec impl fe-mark-item-bought#mark-bought-action */}
              <input
                className="shopping-list__checkbox"
                type="checkbox"
                checked={item.bought}
                onChange={() => handleToggleBought(item)}
                aria-label={`Mark ${item.name} as bought`}
              />
              {/* @spec impl fe-mark-item-bought#distinguish-bought-items */}
              <p
                className={
                  item.bought
                    ? "shopping-list__item-name shopping-list__item-name--bought"
                    : "shopping-list__item-name"
                }
              >
                {item.name}
              </p>
              {/* @spec impl fe-delete-shopping-item#delete-item-action */}
              <button
                className="shopping-list__delete"
                type="button"
                onClick={() => handleDeleteItem(item.id)}
              >
                Delete
              </button>
            </div>
          ))}
        </div>
      )}
      {items.length > 0 && (
        // @spec impl fe-clear-shopping-list#clear-list-action
        <button
          className="shopping-list__clear"
          type="button"
          onClick={handleClearAll}
        >
          Clear all
        </button>
      )}
    </div>
  );
}
