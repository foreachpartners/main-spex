import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import ApiTest from "./api-test";

describe("ApiTest", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders the api response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ text: () => Promise.resolve("hello from api") }),
    );

    render(<ApiTest />);

    expect(await screen.findByText("hello from api")).toBeInTheDocument();
  });
});
