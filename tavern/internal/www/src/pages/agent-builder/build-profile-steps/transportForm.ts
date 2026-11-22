import { SupportedTransports } from "../../../utils/enums";
import { DEFAULT_TRANSPORT_INTERVAL, DEFAULT_TRANSPORT_JITTER, TransportConfig, TransportTypeValue } from "../types";

export type RecordType = "txt" | "a" | "aaaa";
export type DohProvider = "system" | "cloudflare" | "google" | "quad9";

export type TransportForm = {
    transport: TransportConfig;
    domain: string;
    recordType: RecordType;
    doh: DohProvider;
    httpProxy: string;
};

const defaults: Record<TransportTypeValue, { uri: string; interval: number; jitter: number }> = {
    [SupportedTransports.GRPC]: { uri: "", interval: DEFAULT_TRANSPORT_INTERVAL, jitter: DEFAULT_TRANSPORT_JITTER },
    [SupportedTransports.HTTP1]: { uri: "", interval: DEFAULT_TRANSPORT_INTERVAL, jitter: DEFAULT_TRANSPORT_JITTER },
    [SupportedTransports.DNS]: { uri: "dns://*", interval: DEFAULT_TRANSPORT_INTERVAL, jitter: DEFAULT_TRANSPORT_JITTER },
    [SupportedTransports.ICMP]: { uri: "", interval: DEFAULT_TRANSPORT_INTERVAL, jitter: DEFAULT_TRANSPORT_JITTER },
    [SupportedTransports.QUIC]: { uri: "", interval: DEFAULT_TRANSPORT_INTERVAL, jitter: DEFAULT_TRANSPORT_JITTER },
    [SupportedTransports.TCP_BIND]: { uri: "", interval: DEFAULT_TRANSPORT_INTERVAL, jitter: DEFAULT_TRANSPORT_JITTER },
};

export const transportUriHint: Record<TransportTypeValue, string> = {
    [SupportedTransports.GRPC]: "Enter a host, or use http:// or https://.",
    [SupportedTransports.HTTP1]: "Enter a host, or use http1:// or https1://.",
    [SupportedTransports.DNS]: "Use dns://* for the system resolver, or dns://8.8.8.8:53 for a specific server.",
    [SupportedTransports.ICMP]: "Enter the IPv4 address or hostname of the ICMP redirector.",
    [SupportedTransports.QUIC]: "Enter a host, or use quic:// or quics:// with an optional port.",
    [SupportedTransports.TCP_BIND]: "Enter a local bind address, or use tcp:// with a port.",
};

export const transportUriPlaceholder: Record<TransportTypeValue, string> = {
    [SupportedTransports.GRPC]: "grpc.example.com",
    [SupportedTransports.HTTP1]: "http1.example.com",
    [SupportedTransports.DNS]: "dns://*",
    [SupportedTransports.ICMP]: "icmp.example.com",
    [SupportedTransports.QUIC]: "quic.example.com",
    [SupportedTransports.TCP_BIND]: "0.0.0.0",
};

/** Add the default scheme and port to a bare host. */
export const normalizeTransportUri = (type: TransportTypeValue, uri: string): string => {
    const value = uri.trim();
    if (!value || type === SupportedTransports.DNS || value.includes("://")) return value;
    // Leave malformed inputs to validation instead of turning them into plausible URIs.
    if (!/^[a-zA-Z0-9.-]+(?::\d{1,5})?$/.test(value)) return value;

    if (type === SupportedTransports.QUIC) return `quics://${value.includes(":") ? value : `${value}:443`}`;
    if (type === SupportedTransports.TCP_BIND) return `tcp://${value.includes(":") ? value : `${value}:9000`}`;
    if (type === SupportedTransports.ICMP) return `icmp://${value}`;
    if (type === SupportedTransports.HTTP1) return `https1://${value}`;
    return `https://${value}`;
};

const parseExtra = (extra?: string): Record<string, string> => {
    if (!extra?.trim()) return {};
    try {
        const parsed = JSON.parse(extra);
        if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};
        return Object.fromEntries(Object.entries(parsed).filter((entry): entry is [string, string] => typeof entry[1] === "string"));
    } catch {
        return {};
    }
};

export const newTransportForm = (type: TransportTypeValue = SupportedTransports.GRPC): TransportForm => ({
    transport: { id: `t${Date.now()}`, type, ...defaults[type] },
    domain: "",
    recordType: "txt",
    doh: "system",
    httpProxy: "",
});

export const transportFormFromConfig = (transport: TransportConfig): TransportForm => {
    const extra = parseExtra(transport.extra);
    return {
        transport: { ...transport },
        domain: extra.domain ?? "",
        recordType: extra.type === "a" || extra.type === "aaaa" ? extra.type : "txt",
        doh: transport.type === SupportedTransports.HTTP1
            ? (extra.doh ? "cloudflare" : "system")
            : (extra.doh === "cloudflare" || extra.doh === "google" || extra.doh === "quad9" ? extra.doh : "system"),
        httpProxy: extra.http_proxy ?? "",
    };
};

export const switchTransportType = (form: TransportForm, type: TransportTypeValue): TransportForm => {
    if (form.transport.type === type) return form;
    return { ...newTransportForm(type), transport: { id: form.transport.id, type, ...defaults[type] } };
};

/** Per-field validation errors for a transport form, keyed by the field they apply to. */
export type TransportFieldErrors = {
    uri?: string;
    domain?: string;
    interval?: string;
    jitter?: string;
    httpProxy?: string;
};

export const transportFieldErrors = (form: TransportForm): TransportFieldErrors => {
    const { type, uri, interval, jitter } = form.transport;
    const value = normalizeTransportUri(type, uri);
    const errors: TransportFieldErrors = {};

    if (!Number.isInteger(interval) || interval < 1) errors.interval = "Enter an interval of at least one second.";

    if (!Number.isFinite(jitter) || jitter < 0 || jitter > 1) errors.jitter = "Enter a jitter between 0% and 100%.";

    if (!value) {
        errors.uri = "Enter a URI.";
    } else if (/[?#;]/.test(value)) {
        errors.uri = "Enter a URI without query parameters, fragments, or semicolons.";
    } else if (type === SupportedTransports.DNS) {
        const match = /^dns:\/\/(\*|[^/:\s]+)(?::(\d{1,5}))?$/.exec(value);
        if (!match || (match[1] === "*" && match[2]) ||
            (match[2] && (Number(match[2]) < 1 || Number(match[2]) > 65535))) errors.uri = "Use dns://* or dns://host[:port].";
    } else if (type === SupportedTransports.ICMP) {
        const match = /^icmp:\/\/([^/:\s]+)$/.exec(value);
        if (!match || (/^\d+(?:\.\d+){3}$/.test(match[1]) &&
            match[1].split(".").some((octet) => Number(octet) > 255))) {
            errors.uri = "Use icmp:// followed by an IPv4 address or hostname.";
        }
    } else if (type === SupportedTransports.TCP_BIND) {
        try {
            const parsed = new URL(value);
            if (parsed.protocol !== "tcp:" || !parsed.hostname || !parsed.port || parsed.port === "0" ||
                (parsed.pathname && parsed.pathname !== "/") || parsed.username || parsed.password) {
                errors.uri = transportUriHint[type];
            }
        } catch {
            errors.uri = transportUriHint[type];
        }
    } else {
        const schemes = type === SupportedTransports.GRPC ? ["http:", "https:"]
            : type === SupportedTransports.QUIC ? ["quic:", "quics:"] : ["http1:", "https1:"];
        try {
            const parsed = new URL(value);
            if (!schemes.includes(parsed.protocol) || !parsed.hostname ||
                (type === SupportedTransports.QUIC && (parsed.pathname !== "" || parsed.username || parsed.password ||
                    parsed.hostname.startsWith("[") || parsed.port === "0"))) {
                errors.uri = transportUriHint[type];
            }
        } catch {
            errors.uri = transportUriHint[type];
        }
    }

    if (type === SupportedTransports.DNS && !errors.uri && !form.domain.trim()) errors.domain = "Enter a base domain.";

    if (!errors.uri && form.httpProxy.trim()) {
        try {
            const proxy = new URL(form.httpProxy.trim());
            if (!["http:", "https:"].includes(proxy.protocol) || !proxy.hostname) errors.httpProxy = "Enter a full HTTP proxy URI.";
        } catch {
            errors.httpProxy = "Enter a full HTTP proxy URI.";
        }
    }

    return errors;
};

/** Collapses `transportFieldErrors` into a single message, for callers that just need overall validity. */
export const transportFormError = (form: TransportForm): string | null => {
    const errors = transportFieldErrors(form);
    return errors.uri ?? errors.domain ?? errors.interval ?? errors.jitter ?? errors.httpProxy ?? null;
};

export const transportConfigFromForm = (form: TransportForm): TransportConfig => {
    const transport = { ...form.transport, uri: normalizeTransportUri(form.transport.type, form.transport.uri) };
    const extra = parseExtra(transport.extra);
    delete extra.domain;
    delete extra.type;
    delete extra.doh;
    delete extra.http_proxy;

    if (transport.type === SupportedTransports.DNS) {
        extra.domain = form.domain.trim();
        if (form.recordType !== "txt") extra.type = form.recordType;
    }
    if (transport.type === SupportedTransports.GRPC || transport.type === SupportedTransports.HTTP1) {
        if (form.doh !== "system") extra.doh = form.doh;
        if (form.httpProxy.trim()) extra.http_proxy = form.httpProxy.trim();
    }
    return { ...transport, extra: Object.keys(extra).length ? JSON.stringify(extra) : "" };
};

export const transportExtraSummary = (transport: TransportConfig): string[] => {
    const extra = parseExtra(transport.extra);
    if (transport.type === SupportedTransports.DNS) {
        const recordType = extra.type === "a" || extra.type === "aaaa" ? extra.type : "txt";
        return [extra.domain && `Domain: ${extra.domain}`, `Record type: ${recordType.toUpperCase()}`].filter(Boolean) as string[];
    }
    if (transport.type === SupportedTransports.GRPC || transport.type === SupportedTransports.HTTP1) {
        return [extra.doh && `DNS over HTTPS: ${extra.doh}`, extra.http_proxy && `HTTP proxy: ${extra.http_proxy}`].filter(Boolean) as string[];
    }
    return [];
};
