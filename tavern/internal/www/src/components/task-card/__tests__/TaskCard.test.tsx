import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TaskNode } from "../../../utils/interfacesQuery";
import TaskCard from "../TaskCard";

vi.mock("../components/TaskMenu", () => ({ TaskMenu: () => null }));
vi.mock("../components/TaskStatusField", () => ({ default: () => null }));
vi.mock("../components/UserFields", () => ({ default: () => null }));
vi.mock("../components/TomeFields", () => ({ default: () => null }));
vi.mock("../components/BeaconFields", () => ({ default: () => null }));
vi.mock("../components/TaskAttributes", () => ({ default: () => null }));
vi.mock("../components/TaskTome", () => ({ default: () => null }));
vi.mock("../components/TaskShells", () => ({ default: () => null }));
vi.mock("../components/TaskProcesses", () => ({ default: () => null }));
vi.mock("../components/TaskFiles", () => ({ default: () => null }));
vi.mock("../components/TaskScreenshots", () => ({ default: ({ taskId }: { taskId: string }) => <div>Task screenshots for {taskId}</div> }));

const task = (screenshotCount: number, error: string | null = null) => ({
    id: "task-1",
    error,
    output: "result",
    screenshots: { totalCount: screenshotCount },
    shells: { edges: [] },
    reportedProcesses: { totalCount: 0 },
    reportedFiles: { totalCount: 0 },
    quest: { name: "Quest", tome: {}, creator: {} },
    beacon: { host: { id: "host-1" } },
} as unknown as TaskNode);

describe("TaskCard screenshot tab", () => {
    it("only shows screenshots when this task reported them", () => {
        const { rerender } = render(<TaskCard task={task(0)} />);
        expect(screen.queryByRole("tab", { name: /Screenshots/ })).not.toBeInTheDocument();

        rerender(<TaskCard task={task(2)} />);
        expect(screen.getByRole("tab", { name: "Screenshots (2)" })).toBeInTheDocument();
    });

    it("selects Screenshots before Output when the task has screenshots", () => {
        render(<TaskCard task={task(2)} />);

        expect(screen.getByRole("tab", { name: "Screenshots (2)" })).toHaveAttribute("aria-selected", "true");
        expect(screen.getByRole("tab", { name: "Output" })).toHaveAttribute("aria-selected", "false");
    });

    it("selects Output when the task has no error or screenshots", () => {
        render(<TaskCard task={task(0)} />);

        expect(screen.getByRole("tab", { name: "Output" })).toHaveAttribute("aria-selected", "true");
    });

    it("keeps Error ahead of Screenshots and selected by default", () => {
        render(<TaskCard task={task(1, "failed")} />);

        const tabs = screen.getAllByRole("tab");
        expect(tabs.map(tab => tab.textContent?.trim())).toEqual(["Attributes", "Tome", "Output", "Error", "Screenshots (1)"]);
        expect(screen.getByRole("tab", { name: "Error" })).toHaveAttribute("aria-selected", "true");
    });
});
