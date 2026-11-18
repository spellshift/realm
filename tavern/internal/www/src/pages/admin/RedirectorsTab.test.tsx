import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { ChakraProvider } from "@chakra-ui/react";
import { useState } from "react";
import { RedirectorsTab } from "./RedirectorsTab";

const query = vi.fn();
const mutation = vi.fn();
const save = vi.fn();
const refetch = vi.fn();

vi.mock("@apollo/client", async (importOriginal) => {
    const original = await importOriginal<typeof import("@apollo/client")>();
    return {
        ...original,
        useQuery: (...args: unknown[]) => query(...args),
        useMutation: (...args: unknown[]) => mutation(...args),
    };
});

const renderTab = () => render(<ChakraProvider><RedirectorsTab /></ChakraProvider>);

describe("RedirectorsTab", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        refetch.mockResolvedValue({});
        save.mockResolvedValue({ data: { setRedirectorEnabled: false } });
        mutation.mockReturnValue([save, { loading: false }]);
        query.mockReturnValue({
            data: { redirectors: [{ id: "9", domains: ["one.example", "two.example"], online: true, enabled: true }] },
            loading: false,
            refetch,
        });
    });

    it("shows all deployment domains and keeps status separate from the enable toggle", () => {
        renderTab();
        expect(screen.getByText("one.example")).toBeInTheDocument();
        expect(screen.getByText("two.example")).toBeInTheDocument();
        expect(screen.getByText("Online")).toBeInTheDocument();
        expect(screen.getByRole("checkbox", { name: /Enable redirector for one.example, two.example/ })).toBeChecked();
        expect(query.mock.calls[0][1]).toMatchObject({ pollInterval: 30000 });
    });

    it("sends one deployment-level disable and refetches server state", async () => {
        renderTab();
        fireEvent.click(screen.getByRole("checkbox", { name: /Enable redirector/ }));
        await waitFor(() => expect(save).toHaveBeenCalledWith({ variables: { redirectorID: "9", enabled: false } }));
        await waitFor(() => expect(refetch).toHaveBeenCalledTimes(1));
        expect(screen.getByRole("checkbox", { name: /Enable redirector/ })).toBeChecked();
    });

    it("keeps server state and shows an error when the toggle request fails", async () => {
        save.mockRejectedValue(new Error("update failed"));
        mutation.mockImplementation(() => {
            const [error, setError] = useState<Error | undefined>();
            return [async (options: unknown) => {
                try {
                    return await save(options);
                } catch (cause) {
                    setError(cause as Error);
                    throw cause;
                }
            }, { loading: false, error }];
        });
        renderTab();
        fireEvent.click(screen.getByRole("checkbox", { name: /Enable redirector/ }));
        await waitFor(() => expect(screen.getByText("update failed")).toBeInTheDocument());
        expect(screen.getByRole("checkbox", { name: /Enable redirector/ })).toBeChecked();
        expect(refetch).not.toHaveBeenCalled();
    });
});
