import { MockedProvider } from "@apollo/client/testing";
import { ChakraProvider } from "@chakra-ui/react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import "@testing-library/jest-dom/vitest";
import TaskScreenshots, { GET_TASK_SCREENSHOTS } from "../TaskScreenshots";

vi.mock("../../../tavern-base-ui/virtualized-card-list", () => ({
    VirtualizedCardList: ({ items, renderCard }: { items: string[]; renderCard: ({ itemId }: { itemId: string }) => JSX.Element }) => (
        <div>{items.map(itemId => <div key={itemId}>{renderCard({ itemId })}</div>)}</div>
    ),
}));

const screenshots = [
    { id: "image-1", name: "first.png", size: 1024, createdAt: "2026-11-07T14:03:00Z" },
    { id: "image-2", name: "second.png", size: 2048, createdAt: "2026-11-07T14:04:00Z" },
];

describe("TaskScreenshots", () => {
    it("previews only screenshots returned for this task and navigates between them", async () => {
        const user = userEvent.setup();
        render(
            <ChakraProvider>
                <MockedProvider addTypename={false} mocks={[{
                    request: { query: GET_TASK_SCREENSHOTS, variables: { taskId: "task-1" } },
                    result: { data: { tasks: { edges: [{ node: { screenshots: { edges: screenshots.map(node => ({ node })) } } }] } } },
                }]}>
                    <MemoryRouter>
                        <TaskScreenshots taskId="task-1" hostId="host-1" />
                    </MemoryRouter>
                </MockedProvider>
            </ChakraProvider>
        );

        expect(await screen.findByRole("button", { name: "Preview first.png" })).toHaveAttribute("aria-current", "true");
        expect(screen.getByAltText("first.png")).toHaveAttribute("src", "/cdn/screenshots/image-1");
        expect(screen.getByRole("button", { name: "Previous screenshot" })).toBeDisabled();

        await user.click(screen.getByRole("button", { name: "Next screenshot" }));
        expect(screen.getByAltText("second.png")).toHaveAttribute("src", "/cdn/screenshots/image-2");
        expect(screen.getByRole("button", { name: "Preview second.png" })).toHaveAttribute("aria-current", "true");
        expect(screen.getByRole("button", { name: "Next screenshot" })).toBeDisabled();
        expect(screen.getByRole("link", { name: "View host screenshots" })).toHaveAttribute("href", "/hosts/host-1?tab=screenshots");
    });
});
