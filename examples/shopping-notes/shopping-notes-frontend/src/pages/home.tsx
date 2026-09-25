import { useEffect, useState, type CSSProperties, type FormEvent } from "react";

interface Item {
  id: string;
  name: string;
  bought: boolean;
}

const API_BASE_URL = "http://localhost:3000";

const containerStyle: CSSProperties = {
  backgroundColor: "#fafafa",
  display: "flex",
  flexDirection: "column",
  gap: "20px",
  alignItems: "flex-start",
  padding: "32px 16px",
  minHeight: "100vh",
};

const titleStyle: CSSProperties = {
  fontWeight: 600,
  color: "#171717",
  fontSize: "22px",
  margin: 0,
};

const addRowStyle: CSSProperties = {
  display: "flex",
  gap: "8px",
  alignItems: "center",
  width: "100%",
};

const inputStyle: CSSProperties = {
  flex: 1,
  border: "1px solid #e0e0e0",
  borderRadius: "8px",
  padding: "10px 12px",
  fontSize: "15px",
  color: "#171717",
};

const addButtonStyle: CSSProperties = {
  backgroundColor: "#3373f2",
  color: "#fff",
  fontWeight: 600,
  fontSize: "14px",
  padding: "10px 16px",
  borderRadius: "8px",
  border: "none",
};

const listStyle: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "10px",
  width: "100%",
};

const itemRowStyle: CSSProperties = {
  display: "flex",
  gap: "12px",
  alignItems: "center",
  width: "100%",
  border: "1px solid #e0e0e0",
  borderRadius: "8px",
  padding: "12px",
  backgroundColor: "#fff",
};

const checkboxStyle: CSSProperties = {
  width: "20px",
  height: "20px",
  margin: 0,
  accentColor: "#3373f2",
};

const itemNameStyle: CSSProperties = {
  flex: 1,
  fontSize: "15px",
  color: "#171717",
  margin: 0,
};

const itemNameBoughtStyle: CSSProperties = {
  ...itemNameStyle,
  color: "#8c8c8c",
  textDecoration: "line-through",
};

const deleteButtonStyle: CSSProperties = {
  background: "none",
  border: "none",
  color: "#d94040",
  fontSize: "13px",
  cursor: "pointer",
};

const emptyStateStyle: CSSProperties = {
  width: "100%",
  backgroundColor: "#fff",
  borderRadius: "8px",
  padding: "60px 0 20px",
  textAlign: "center",
  color: "#8c8c8c",
  fontSize: "15px",
};

const clearButtonStyle: CSSProperties = {
  backgroundColor: "#fff",
  border: "1px solid #e0e0e0",
  borderRadius: "8px",
  padding: "10px 16px",
  fontWeight: 600,
  fontSize: "14px",
  color: "#171717",
  cursor: "pointer",
};

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

    setItems((existing) => existing.map((existingItem) => (existingItem.id === updated.id ? updated : existingItem)));
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
    <div style={containerStyle}>
      <p style={titleStyle}>Shopping notes</p>
      {/* @spec impl fe-add-shopping-item#add-item-input */}
      <form style={addRowStyle} onSubmit={handleAddItem}>
        <input
          style={inputStyle}
          type="text"
          placeholder="Add an item"
          value={newItemName}
          onChange={(event) => setNewItemName(event.target.value)}
        />
        <button style={addButtonStyle} type="submit">
          Add
        </button>
      </form>
      {items.length === 0 ? (
        // @spec impl fe-view-shopping-list#empty-state
        <div style={emptyStateStyle}>No items yet</div>
      ) : (
        // @spec impl fe-view-shopping-list#display-item-list
        <div style={listStyle}>
          {items.map((item) => (
            <div key={item.id} style={itemRowStyle}>
              {/* @spec impl fe-mark-item-bought#mark-bought-action */}
              <input
                style={checkboxStyle}
                type="checkbox"
                checked={item.bought}
                onChange={() => handleToggleBought(item)}
                aria-label={`Mark ${item.name} as bought`}
              />
              {/* @spec impl fe-mark-item-bought#distinguish-bought-items */}
              <p style={item.bought ? itemNameBoughtStyle : itemNameStyle}>{item.name}</p>
              {/* @spec impl fe-delete-shopping-item#delete-item-action */}
              <button style={deleteButtonStyle} type="button" onClick={() => handleDeleteItem(item.id)}>
                Delete
              </button>
            </div>
          ))}
        </div>
      )}
      {items.length > 0 && (
        // @spec impl fe-clear-shopping-list#clear-list-action
        <button style={clearButtonStyle} type="button" onClick={handleClearAll}>
          Clear all
        </button>
      )}
    </div>
  );
}
