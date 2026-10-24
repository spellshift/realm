import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ChakraProvider } from "@chakra-ui/react";
import TargetSelectionTable from "../TargetSelectionTable";
import { COMPILE_TARGETS } from "../../types";

vi.mock("@tanstack/react-virtual", async () => {
    const React = await import("react");
    return {
        useVirtualizer: ({ count }: { count: number }) => {
            const virtualizer = React.useMemo(() => ({
                getVirtualItems: () =>
                    Array.from({ length: count }, (_, index) => ({
                        index,
                        key: index,
                        start: index * 48,
                        size: 48,
                        end: (index + 1) * 48,
                    })),
                getTotalSize: () => count * 48,
                measureElement: () => {},
                range: { startIndex: 0, endIndex: Math.max(0, count - 1) },
            }), [count]);
            return virtualizer;
        },
    };
});

describe("TargetSelectionTable", () => {
    it("renders target platforms, formats, and descriptions", () => {
        const onSelect = vi.fn();
        render(
            <ChakraProvider>
                <TargetSelectionTable
                    targets={COMPILE_TARGETS}
                    selectedTargetId={COMPILE_TARGETS[0].id}
                    onSelectTarget={onSelect}
                />
            </ChakraProvider>
        );

        expect(screen.getByText("Linux")).toBeInTheDocument();
        expect(screen.getByText("macOS")).toBeInTheDocument();
        expect(screen.getAllByText("Windows").length).toBeGreaterThan(0);
        expect(screen.getByText("ELF binary, glibc/musl")).toBeInTheDocument();
    });

    it("shows a checkmark for the selected target and an empty ring for others", () => {
        const onSelect = vi.fn();
        const { container } = render(
            <ChakraProvider>
                <TargetSelectionTable
                    targets={COMPILE_TARGETS}
                    selectedTargetId={COMPILE_TARGETS[0].id}
                    onSelectTarget={onSelect}
                />
            </ChakraProvider>
        );

        // Selected row shows a filled checkmark icon.
        expect(container.querySelectorAll("svg.w-6.h-6").length).toBe(1);
        // Unselected rows show an empty ring span.
        expect(container.querySelectorAll("span.rounded-full.border-2").length).toBe(
            COMPILE_TARGETS.length - 1
        );
    });

    it("calls onSelectTarget when a row is clicked", () => {
        const onSelect = vi.fn();
        render(
            <ChakraProvider>
                <TargetSelectionTable
                    targets={COMPILE_TARGETS}
                    selectedTargetId={COMPILE_TARGETS[0].id}
                    onSelectTarget={onSelect}
                />
            </ChakraProvider>
        );

        const rows = screen.getAllByRole("row").slice(1);
        fireEvent.click(rows[1]);
        expect(onSelect).toHaveBeenCalledWith(COMPILE_TARGETS[1]);
    });
});
