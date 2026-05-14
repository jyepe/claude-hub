import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import { ContextMeter } from "./ContextMeter";

describe("ContextMeter", () => {
  it("renders with ok class under 60%", () => {
    const { container } = render(<ContextMeter tokens={50_000} window={200_000} />);
    expect(container.querySelector(".meter-fill-ok")).not.toBeNull();
  });
  it("renders with warn class between 60 and 85%", () => {
    const { container } = render(<ContextMeter tokens={140_000} window={200_000} />);
    expect(container.querySelector(".meter-fill-warn")).not.toBeNull();
  });
  it("renders with danger class above 85%", () => {
    const { container } = render(<ContextMeter tokens={180_000} window={200_000} />);
    expect(container.querySelector(".meter-fill-danger")).not.toBeNull();
  });
});
