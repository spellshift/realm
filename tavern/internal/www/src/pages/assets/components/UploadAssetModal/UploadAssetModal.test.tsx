import { render } from "../../../../test-utils";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import UploadAssetModal from "./UploadAssetModal";
import { MAX_FILE_SIZE } from "./upload";
import { vi, describe, it, expect } from "vitest";

describe("UploadAssetModal", () => {
    it("displays error when file size exceeds limit", async () => {
        const user = userEvent.setup();
        const setOpen = vi.fn();
        const onUploadSuccess = vi.fn();

        render(
            <UploadAssetModal isOpen={true} setOpen={setOpen} onUploadSuccess={onUploadSuccess} />
        );

        // Create a dummy file
        const file = new File(["dummy content"], "large-file.txt", { type: "text/plain" });
        // Mock the size property to exceed the limit
        Object.defineProperty(file, 'size', { value: MAX_FILE_SIZE + 1 });

        const input = screen.getByTestId("file-upload-input");
        await user.upload(input, file);

        const errorMessage = await screen.findByText("File size exceeds 100MB limit");
        expect(errorMessage).toBeInTheDocument();
    });

    it("does not display error for valid file size", async () => {
        const user = userEvent.setup();
        const setOpen = vi.fn();
        const onUploadSuccess = vi.fn();

        render(
            <UploadAssetModal isOpen={true} setOpen={setOpen} onUploadSuccess={onUploadSuccess} />
        );

        // Create a valid file
        const file = new File(["valid content"], "valid-file.txt", { type: "text/plain" });
        // Mock the size property to be within limit
        Object.defineProperty(file, 'size', { value: MAX_FILE_SIZE - 1 });

        const input = screen.getByTestId("file-upload-input");
        await user.upload(input, file);

        expect(screen.queryByText("File size exceeds 100MB limit")).not.toBeInTheDocument();
        expect(screen.getByText("valid-file.txt")).toBeInTheDocument();
    });
});
