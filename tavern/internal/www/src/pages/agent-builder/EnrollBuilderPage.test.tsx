import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MockedProvider } from "@apollo/client/testing";
import { BrowserRouter } from "react-router-dom";
import { describe, expect, it } from "vitest";
import "@testing-library/jest-dom";
import { EnrollBuilderPage } from "./EnrollBuilderPage";
import { REGISTER_BUILDER } from "./queries";

describe("EnrollBuilderPage", () => {
    it("requires a CSR and submits it with the registration request", async () => {
        const csr = "-----BEGIN CERTIFICATE REQUEST-----\ntest\n-----END CERTIFICATE REQUEST-----";
        const mocks = [{
            request: {
                query: REGISTER_BUILDER,
                variables: {
                    csr,
                    input: {
                        supportedTargets: ["PLATFORM_LINUX", "PLATFORM_WINDOWS"],
                        upstream: "http://127.0.0.1:8000",
                        pollInterval: 5,
                        executor: "docker",
                    },
                },
            },
            result: { data: { registerBuilder: { builder: { id: "1", identifier: "abc" }, config: "mtls_key_file: builder.key" } } },
        }];
        render(<MockedProvider mocks={mocks} addTypename={false}><BrowserRouter><EnrollBuilderPage /></BrowserRouter></MockedProvider>);
        const submit = screen.getByRole("button", { name: "Configure" });
        expect(submit).toBeDisabled();
        fireEvent.change(screen.getByLabelText("Certificate request (PEM)"), { target: { value: csr } });
        expect(submit).toBeEnabled();
        fireEvent.click(submit);
        await waitFor(() => expect(screen.getByText(/Save this configuration beside/)).toBeInTheDocument());
    });
});
