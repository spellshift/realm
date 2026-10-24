import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ChakraProvider } from "@chakra-ui/react";
import TargetSelectionTable from "../TargetSelectionTable";
import { COMPILE_TARGETS } from "../../types";

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
