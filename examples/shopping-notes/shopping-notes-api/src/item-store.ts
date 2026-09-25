import { randomUUID } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";

export interface Item {
  id: string;
  name: string;
  bought: boolean;
}

const DEFAULT_DATA_FILE = fileURLToPath(new URL("../data/items.json", import.meta.url));

export class ItemStore {
  private items: Item[];
  private readonly filePath: string;

  constructor(filePath: string = process.env.SHOPPING_NOTES_API_DATA_FILE ?? DEFAULT_DATA_FILE) {
    this.filePath = filePath;
    this.items = this.readFromDisk();
  }

  private readFromDisk(): Item[] {
    if (!existsSync(this.filePath)) {
      return [];
    }

    const raw = readFileSync(this.filePath, "utf-8");
    return raw.trim() ? (JSON.parse(raw) as Item[]) : [];
  }

  private writeToDisk(): void {
    mkdirSync(dirname(this.filePath), { recursive: true });
    writeFileSync(this.filePath, JSON.stringify(this.items, null, 2));
  }

  list(): Item[] {
    return this.items;
  }

  create(name: string): Item {
    // @spec impl api-add-shopping-item#assign-unique-id
    const item: Item = { id: randomUUID(), name, bought: false };
    this.items.push(item);
    // @spec impl api-add-shopping-item#persist-created-item
    this.writeToDisk();
    return item;
  }

  update(id: string, bought: boolean): Item | undefined {
    const item = this.items.find((existing) => existing.id === id);
    if (!item) {
      return undefined;
    }

    item.bought = bought;
    // @spec impl api-mark-item-bought#persist-bought-status
    this.writeToDisk();
    return item;
  }

  delete(id: string): boolean {
    const index = this.items.findIndex((existing) => existing.id === id);
    if (index === -1) {
      return false;
    }

    this.items.splice(index, 1);
    // @spec impl api-delete-shopping-item#persist-deletion
    this.writeToDisk();
    return true;
  }

  clear(): void {
    this.items = [];
    // @spec impl api-clear-shopping-list#persist-cleared-list
    this.writeToDisk();
  }
}
