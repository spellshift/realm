import { ApolloClient, ApolloLink, ApolloProvider, InMemoryCache, Observable } from "@apollo/client";
import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { HostContext } from "../../../../context/HostContext";
import { useScreenshotQuestFormData } from "../useScreenshotQuestFormData";
import { GET_TOME_IDS_QUERY } from "../../../../utils/queries";

describe("useScreenshotQuestFormData", () => {
    it("prefills the Report screenshot tome and an online beacon on this host", async () => {
        const requests: Array<{ name: string | undefined; variables: Record<string, any> }> = [];
        const client = new ApolloClient({
            cache: new InMemoryCache(),
            link: new ApolloLink(operation => new Observable(observer => {
                requests.push({ name: operation.operationName, variables: operation.variables });
                setTimeout(() => {
                    observer.next({ data: operation.operationName === "GetTomeIds"
                        ? { tomes: { edges: [{ node: { id: "tome-1", name: "Report screenshot", paramDefs: '[{"name":"quality","label":"Quality","type":"string","placeholder":"","value":"old"}]' } }] } }
                        : { beacons: { edges: [{ node: { id: "beacon-1", principal: "user", transport: "GRPC" } }] } },
                    });
                    observer.complete();
                }, 0);
            })),
        });
        const wrapper = ({ children }: { children: React.ReactNode }) => (
            <ApolloProvider client={client}>
                <HostContext.Provider value={{ data: { id: "host-1", name: "workstation" } as any, loading: false, error: undefined, taskCount: 0, totalShellCount: 0, activeShellCount: 0, totalPortalCount: 0, activePortalCount: 0 }}>
                    {children}
                </HostContext.Provider>
            </ApolloProvider>
        );

        const { result } = renderHook(() => useScreenshotQuestFormData(), { wrapper });
        await waitFor(() => expect(client.readQuery({ query: GET_TOME_IDS_QUERY, variables: { where: { name: "Report screenshot" } } })).not.toBeNull());
        let formData;
        await waitFor(async () => {
            formData = await result.current.fetchFormData();
            expect(formData?.tomeId).toBe("tome-1");
        });

        expect(requests[0].variables).toEqual({ where: { name: "Report screenshot" } });
        const beaconRequest = requests.find(request => request.name === "GetOnlineHostBeacons");
        expect(beaconRequest?.variables.where.hasHostWith).toEqual({ id: "host-1" });
        expect(new Date(beaconRequest?.variables.where.nextSeenAtGTE).getTime()).toBeGreaterThan(Date.now() - 35000);
        expect(formData).toMatchObject({
            name: "Populate workstation Report screenshot",
            beacons: ["beacon-1"],
            params: [{ name: "quality", value: "" }],
            initialStep: 1,
            initialFilters: { tomeMultiSearch: "Report screenshot", beaconFields: [{ id: "host-1", kind: "host" }] },
        });
    }, 15000);
});
