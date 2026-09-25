import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import Home from "./home";

describe("Home", () => {
  it("renders hello from frontend", () => {
    render(<Home />);

    expect(screen.getByText("hello from frontend")).toBeInTheDocument();
  });
});
