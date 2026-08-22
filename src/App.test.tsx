import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";
import App from "./App";

vi.mock("./lib/ipc", () => ({
  ipc: {
    listScans: vi.fn().mockResolvedValue([]),
    licenseStatus: vi.fn().mockResolvedValue({ state: "unlicensed", lastVerifiedAt: null, graceExpiresAt: null }),
  },
  errorMessage: () => "Safe error",
}));

describe("App", () => {
  it("loads desktop state and navigates through semantic controls", async () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "Cloud posture, without the noise." })).toBeVisible();
    await waitFor(() => expect(screen.getByLabelText("License unlicensed")).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: "Inventory" }));
    expect(screen.getByRole("heading", { name: "Inventory" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Choose a snapshot" })).toBeEnabled();
  });
});
