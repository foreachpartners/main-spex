import "@testing-library/jest-dom/vitest";
import "./style.css";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

afterEach(() => {
  cleanup();
});
