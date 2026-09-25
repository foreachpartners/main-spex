import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import App from "./app";

describe("App", () => {
  it("renders the not-found page for an unmatched route", () => {
    // @spec test fe-not-found-page#unmatched-route-shows-not-found-page
    render(
      <MemoryRouter initialEntries={["/does-not-exist"]}>
        <App />
      </MemoryRouter>,
    );

    expect(screen.getByText("Page not found")).toBeInTheDocument();
  });
});
