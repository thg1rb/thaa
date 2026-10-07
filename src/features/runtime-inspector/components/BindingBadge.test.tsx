import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { BindingBadge } from "./BindingBadge";

afterEach(cleanup);

describe("BindingBadge", () => {
  it.each([
    ["loopbackOnly", "Loopback only"],
    ["wildcardIpv4", "All IPv4 interfaces"],
    ["wildcardIpv6", "All IPv6 interfaces"],
    ["specificAddress", "Specific address"],
    ["unknown", "Address unknown"],
  ] as const)("labels %s as %s", (binding, label) => {
    render(<BindingBadge binding={binding} localAddress="192.0.2.8" />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it("explains that an IPv4 wildcard does not prove remote reachability", () => {
    render(<BindingBadge binding="wildcardIpv4" localAddress="0.0.0.0" />);
    expect(screen.getByText("All IPv4 interfaces")).toHaveAttribute(
      "aria-description",
      expect.stringContaining("does not prove LAN or Internet reachability"),
    );
  });

  it("does not infer dual-stack behavior for an IPv6 wildcard", () => {
    render(<BindingBadge binding="wildcardIpv6" localAddress="::" />);
    expect(screen.getByText("All IPv6 interfaces")).toHaveAttribute(
      "aria-description",
      expect.stringContaining("IPv4 dual-stack behavior"),
    );
  });

  it("keeps the address in specific-bind help text", () => {
    render(<BindingBadge binding="specificAddress" localAddress="192.0.2.8" />);
    expect(screen.getByText("Specific address")).toHaveAttribute(
      "title",
      expect.stringContaining("192.0.2.8"),
    );
  });
});
