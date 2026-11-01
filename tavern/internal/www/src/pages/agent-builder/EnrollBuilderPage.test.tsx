import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { MemoryRouter } from "react-router-dom";
import { print } from "graphql";
import EnrollBuilderPage from "./EnrollBuilderPage";

const createToken = vi.fn();
const useQuery = vi.fn();
const useMutation = vi.fn();
vi.mock("@apollo/client", async (importOriginal) => {
    const original = await importOriginal<typeof import("@apollo/client")>();
    return {
        ...original,
        useMutation: (...args: unknown[]) => {
            useMutation(...args);
            return [createToken, { loading: false }];
        },
        useQuery: (...args: unknown[]) => useQuery(...args),
    };
});

const renderPage = () => render(<MemoryRouter><EnrollBuilderPage /></MemoryRouter>);

describe("EnrollBuilderPage", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("generates only on click and shows a copyable token and command without polling", async () => {
        createToken.mockResolvedValue({ data: { createBuilderEnrollmentToken: "test-token" } });
        renderPage();
        expect(createToken).not.toHaveBeenCalled();
        expect(screen.queryByText("test-token")).not.toBeInTheDocument();

        fireEvent.click(screen.getByRole("button", { name: "Generate" }));
        await waitFor(() => expect(screen.getByText("test-token")).toBeInTheDocument());
        expect(screen.getByText(/REALM_BUILDER_ENROLLMENT_TOKEN=/)).toHaveTextContent("REALM_BUILDER_ENROLLMENT_TOKEN='test-token' go run ./tavern builder");
        expect(screen.getAllByRole("button", { name: "Copy code" })).toHaveLength(2);
        expect(screen.getByRole("button", { name: "Done" })).toBeEnabled();
        expect(createToken).toHaveBeenCalledTimes(1);
        expect(print(useMutation.mock.calls[0][0])).toContain("createBuilderEnrollmentToken");
        expect(print(useMutation.mock.calls[0][0])).not.toContain("registerBuilder");
        expect(useQuery).not.toHaveBeenCalled();
        expect(screen.queryByText(/Waiting for the builder/)).not.toBeInTheDocument();
    });

    it("shows an error and allows retry", async () => {
        createToken.mockRejectedValueOnce(new Error("temporary failure"));
        createToken.mockResolvedValueOnce({ data: { createBuilderEnrollmentToken: "second-token" } });
        renderPage();
        fireEvent.click(screen.getByRole("button", { name: "Generate" }));
        await waitFor(() => expect(screen.getByText("temporary failure")).toBeInTheDocument());
        fireEvent.click(screen.getByRole("button", { name: "Generate" }));
        await waitFor(() => expect(screen.getByText("second-token")).toBeInTheDocument());
        expect(screen.queryByText("temporary failure")).not.toBeInTheDocument();
        expect(createToken).toHaveBeenCalledTimes(2);
    });
});
