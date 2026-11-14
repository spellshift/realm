import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_TRANSPORT } from "../../types";
import { ReviewStep } from "../ReviewStep";

describe("ReviewStep", () => {
    it("shows transport settings in the shared read-only list", () => {
        render(<ReviewStep
            name="Example" description="" buildImage="builder:latest"
            transports={[{ ...DEFAULT_TRANSPORT, extra: JSON.stringify({ doh: "cloudflare" }) }]}
            tomes={[]} hasPersistenceTome
            isEditing={false} saving={false} canSave onBack={vi.fn()} onSave={vi.fn()}
        />);

        expect(screen.getByRole("table")).toBeInTheDocument();
        expect(screen.getByRole("columnheader", { name: "URI" })).toBeInTheDocument();
        expect(screen.getByText(DEFAULT_TRANSPORT.uri)).toBeInTheDocument();
        expect(screen.getByText("DNS over HTTPS: cloudflare")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Edit" })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /Remove.*transport/ })).not.toBeInTheDocument();
    });
});
