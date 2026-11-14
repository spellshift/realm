import { describe, expect, it } from "vitest";
import { SupportedTransports } from "../../../../utils/enums";
import { DEFAULT_TRANSPORT, TRANSPORT_TYPES, TransportConfig } from "../../types";
import {
    newTransportForm, switchTransportType, transportConfigFromForm, transportExtraSummary,
    transportFormError, transportFormFromConfig,
} from "../transportForm";

describe("transport form", () => {
    it("requires a DNS domain and saves the selected record type", () => {
        const form = newTransportForm(SupportedTransports.DNS);
        expect(form.transport.uri).toBe("dns://*");
        expect(form.transport.interval).toBe(30);
        expect(transportFormError(form)).toMatch(/base domain/);
        form.domain = "c2.example.com";
        form.recordType = "aaaa";
        expect(transportFormError(form)).toBeNull();
        const saved = transportConfigFromForm(form);
        expect(JSON.parse(saved.extra ?? "")).toEqual({ domain: "c2.example.com", type: "aaaa" });
        expect(transportExtraSummary(saved)).toEqual(["Domain: c2.example.com", "Record type: AAAA"]);
    });

    it("round trips gRPC DNS and proxy options while keeping unknown extra keys", () => {
        const original: TransportConfig = {
            id: "t1", type: SupportedTransports.GRPC, uri: "https://c2.example.com", interval: 5,
            extra: '{"doh":"google","http_proxy":"http://127.0.0.1:3128","custom":"keep"}',
        };
        const form = transportFormFromConfig(original);
        expect(form.doh).toBe("google");
        expect(form.httpProxy).toBe("http://127.0.0.1:3128");
        expect(JSON.parse(transportConfigFromForm(form).extra ?? "")).toEqual(JSON.parse(original.extra ?? ""));
        const switched = switchTransportType(form, SupportedTransports.DNS);
        expect(switched.transport.uri).toBe("dns://*");
        expect(switched.transport.extra).toBeUndefined();
        expect(switched.doh).toBe("system");
    });

    it("starts every transport type at a 30 second interval", () => {
        for (const { value: type } of TRANSPORT_TYPES) {
            expect(newTransportForm(type).transport.interval).toBe(30);
            expect(switchTransportType(newTransportForm(), type).transport.interval).toBe(30);
        }
        expect(DEFAULT_TRANSPORT.interval).toBe(30);
    });

    it("checks the URI scheme for each type", () => {
        const grpc = newTransportForm(SupportedTransports.GRPC);
        grpc.transport.uri = "https1://c2.example.com";
        expect(transportFormError(grpc)).toMatch(/http:\/\//);
        const http = newTransportForm(SupportedTransports.HTTP1);
        http.transport.uri = "https1://c2.example.com";
        expect(transportFormError(http)).toBeNull();
        http.httpProxy = "socks5://127.0.0.1:1080";
        expect(transportFormError(http)).toMatch(/HTTP proxy URI/);
        http.httpProxy = "";
        http.transport.uri = "https1://c2.example.com?interval=10";
        expect(transportFormError(http)).toMatch(/query parameters/);
        const icmp = newTransportForm(SupportedTransports.ICMP);
        icmp.transport.uri = "icmp://999.1.1.1";
        expect(transportFormError(icmp)).toMatch(/IPv4/);
        icmp.transport.uri = "icmp://redirector.example.com";
        expect(transportFormError(icmp)).toBeNull();
    });
});
