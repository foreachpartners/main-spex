import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import NotFound from "./not-found";

describe("NotFound", () => {
  it("links back to the main page", () => {
    // @spec test fe-not-found-page#not-found-page-links-to-main-page
    render(
      <MemoryRouter>
        <NotFound />
      </MemoryRouter>,
    );

    expect(screen.getByRole("link", { name: "Back to main page" })).toHaveAttribute("href", "/");
  });
});
