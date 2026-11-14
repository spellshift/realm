import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SupportedTransports } from "../../../../utils/enums";
import { DEFAULT_TRANSPORT, TransportConfig } from "../../types";
import { TransportsStep } from "../TransportsStep";

const Harness = () => {
    const [transports, setTransports] = useState<TransportConfig[]>([DEFAULT_TRANSPORT]);
    return <>
        <TransportsStep transports={transports} setTransports={setTransports} onBack={vi.fn()} onNext={vi.fn()} />
        <output data-testid="saved-transports">{JSON.stringify(transports)}</output>
    </>;
};

describe("TransportsStep", () => {
    it("edits a saved transport in the separate W form", () => {
        render(<Harness />);
        fireEvent.click(screen.getByRole("button", { name: "Edit gRPC transport" }));

        expect(screen.getByRole("region", { name: "Edit transport" })).toBeInTheDocument();
        expect(screen.getByRole("heading", { name: "Edit gRPC transport" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "System DNS" })).toHaveAttribute("aria-pressed", "true");
        expect(screen.getByRole("button", { name: "continue transports step" })).toBeDisabled();
        expect(screen.queryByRole("button", { name: /Advanced/ })).not.toBeInTheDocument();
        expect(screen.getByRole("textbox", { name: "URI" })).toBeRequired();
        expect(screen.getByRole("spinbutton", { name: "Interval (s)" })).toBeRequired();
        expect(screen.getByText("Transport type").textContent).toBe("Transport type*");
        expect(screen.getByRole("textbox", { name: "HTTP proxy" })).not.toBeRequired();

        fireEvent.change(screen.getByRole("textbox", { name: "URI" }), { target: { value: "https://c2.example.com:8443" } });
        fireEvent.click(screen.getByRole("button", { name: "Save transport" }));

        const saved = JSON.parse(screen.getByTestId("saved-transports").textContent ?? "[]") as TransportConfig[];
        expect(saved[0].uri).toBe("https://c2.example.com:8443");
        expect(screen.getByRole("button", { name: "continue transports step" })).toBeEnabled();
    });

    it("adds DNS from the separate form with its required domain and selected record type", () => {
        render(<Harness />);
        fireEvent.click(screen.getByRole("button", { name: "Add transport" }));
        fireEvent.click(screen.getByRole("button", { name: "DNS" }));
        expect(screen.getByRole("textbox", { name: "Base domain" })).toBeRequired();
        expect(screen.getByText("Record type").textContent).toBe("Record type*");
        expect(screen.getByRole("button", { name: "Add transport" })).toBeDisabled();
        fireEvent.change(screen.getByPlaceholderText("c2.example.com"), { target: { value: "c2.example.com" } });
        fireEvent.click(screen.getByRole("button", { name: "AAAA" }));
        fireEvent.click(screen.getByRole("button", { name: "Add transport" }));
        const saved = JSON.parse(screen.getByTestId("saved-transports").textContent ?? "[]") as TransportConfig[];
        expect(saved).toHaveLength(2);
        expect(saved[1].type).toBe(SupportedTransports.DNS);
        expect(JSON.parse(saved[1].extra ?? "")).toEqual({ domain: "c2.example.com", type: "aaaa" });
    });
});
