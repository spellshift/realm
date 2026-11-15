import { Dispatch, SetStateAction, useState } from "react";
import { FormControl, FormErrorMessage, FormLabel, Heading, Input } from "@chakra-ui/react";
import { Plus } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { SupportedTransports } from "../../../utils/enums";
import { TRANSPORT_TYPES, TransportConfig, TransportTypeValue } from "../types";
import { SegmentedControl } from "./SegmentedControl";
import { TransportList, transportLabel } from "./TransportList";
import {
    DohProvider, RecordType, TransportFieldErrors, TransportForm, newTransportForm, switchTransportType,
    transportConfigFromForm, transportFieldErrors, transportFormError, transportFormFromConfig,
    transportUriPlaceholder,
} from "./transportForm";

type Props = {
    transports: TransportConfig[];
    setTransports: Dispatch<SetStateAction<TransportConfig[]>>;
    onBack: () => void;
    onNext: () => void;
};

const dohOptions: Array<{ label: string; value: DohProvider }> = [
    { label: "System DNS", value: "system" },
    { label: "Cloudflare", value: "cloudflare" },
    { label: "Google", value: "google" },
    { label: "Quad9", value: "quad9" },
];
const recordTypes: Array<{ label: string; value: RecordType }> = [
    { label: "TXT", value: "txt" }, { label: "A", value: "a" }, { label: "AAAA", value: "aaaa" },
];

const RequiredMark = () => <span className="text-red-500 ml-1" aria-hidden="true">*</span>;

type FieldProps = {
    id: string;
    label: string;
    value: string | number;
    onChange: (value: string) => void;
    onBlur?: () => void;
    error?: string;
    placeholder?: string;
    type?: "text" | "number";
    min?: number;
    required?: boolean;
};

const TransportField = ({ id, label, value, onChange, onBlur, error, placeholder, type, min, required }: FieldProps) => (
    <FormControl isInvalid={!!error} className="min-w-0">
        <FormLabel htmlFor={id} className="mb-1 text-sm font-medium text-gray-700">{label}{required && <RequiredMark />}</FormLabel>
        <Input
            id={id}
            name={id}
            size="sm"
            colorScheme="purple"
            value={value}
            placeholder={placeholder}
            type={type}
            min={min}
            required={required}
            onChange={(event) => onChange(event.target.value)}
            onBlur={onBlur}
        />
        {error && <FormErrorMessage>{error}</FormErrorMessage>}
    </FormControl>
);

type EditorProps = {
    form: TransportForm;
    editing: boolean;
    error: string | null;
    setForm: Dispatch<SetStateAction<TransportForm | null>>;
    onTypeChange: (type: TransportTypeValue) => void;
    onCancel: () => void;
    onSave: () => void;
};

const TransportEditor = ({ form, editing, error, setForm, onTypeChange, onCancel, onSave }: EditorProps) => {
    // Pre-filled fields (editing an existing transport) start "touched" so an
    // existing problem is visible immediately; a blank new form stays quiet
    // until the user actually leaves each field.
    const [touched, setTouched] = useState<Record<keyof TransportFieldErrors, boolean>>(
        () => ({ uri: editing, domain: editing, interval: editing, httpProxy: editing })
    );
    const touch = (field: keyof TransportFieldErrors) => setTouched((current) => ({ ...current, [field]: true }));
    const errors = transportFieldErrors(form);
    const fieldError = (field: keyof TransportFieldErrors) => (touched[field] ? errors[field] : undefined);

    const update = (change: (current: TransportForm) => TransportForm) => {
        setForm((current) => current ? change(current) : null);
    };
    const type = form.transport.type;
    const isDNS = type === SupportedTransports.DNS;
    const supportsProxy = type === SupportedTransports.GRPC || type === SupportedTransports.HTTP1;

    return (
        <section className="flex flex-col gap-4 rounded-lg border border-gray-200 bg-white p-5 shadow-sm" aria-label={editing ? "Edit transport" : "New transport"}>
            <h3 className="text-base font-semibold text-gray-900">
                {editing ? `Edit ${transportLabel(type)} transport` : "New transport"}
            </h3>

            <div>
                <span className="mb-1 block text-sm font-medium text-gray-700">Transport type<RequiredMark /></span>
                <SegmentedControl label="Transport type" options={TRANSPORT_TYPES} value={type} onChange={onTypeChange} />
            </div>

            <div className={isDNS ? "grid grid-cols-1 gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_7rem]" : "grid grid-cols-1 gap-4 sm:grid-cols-[minmax(0,1fr)_7rem]"}>
                <TransportField id="transportUri" label="URI" required value={form.transport.uri}
                    placeholder={transportUriPlaceholder[type]} error={fieldError("uri")} onBlur={() => touch("uri")}
                    onChange={(value) => update((current) => ({ ...current, transport: { ...current.transport, uri: value } }))} />
                {isDNS && <TransportField id="transportDomain" label="Base domain" required value={form.domain}
                    placeholder="c2.example.com" error={fieldError("domain")} onBlur={() => touch("domain")}
                    onChange={(value) => update((current) => ({ ...current, domain: value }))} />}
                <TransportField id="transportInterval" label="Interval (s)" required type="number" min={1} value={form.transport.interval}
                    error={fieldError("interval")} onBlur={() => touch("interval")}
                    onChange={(value) => update((current) => ({ ...current, transport: { ...current.transport, interval: Number(value) } }))} />
            </div>

            {isDNS && <div>
                <span className="mb-1 block text-sm font-medium text-gray-700">Record type<RequiredMark /></span>
                <SegmentedControl label="DNS record type" options={recordTypes} value={form.recordType}
                    onChange={(value) => update((current) => ({ ...current, recordType: value }))} />
            </div>}

            {supportsProxy && <div className="grid grid-cols-1 gap-4">
                <div>
                    <span className="mb-1 block text-sm font-medium text-gray-700">DNS over HTTPS</span>
                    <SegmentedControl label="DNS over HTTPS" options={type === SupportedTransports.GRPC ? dohOptions : dohOptions.slice(0, 2)}
                        value={form.doh} onChange={(value) => update((current) => ({ ...current, doh: value }))} />
                </div>
                <TransportField id="transportProxy" label="HTTP proxy" value={form.httpProxy}
                    placeholder="http://127.0.0.1:3128" error={fieldError("httpProxy")} onBlur={() => touch("httpProxy")}
                    onChange={(value) => update((current) => ({ ...current, httpProxy: value }))} />
            </div>}

            <div className="flex flex-wrap items-center justify-end gap-2 pt-1">
                <Button buttonVariant="ghost" onClick={onCancel}>Cancel</Button>
                <Button onClick={onSave} disabled={!!error}>{editing ? "Save transport" : "Add transport"}</Button>
            </div>
        </section>
    );
};

export const TransportsStep = ({ transports, setTransports, onBack, onNext }: Props) => {
    const [form, setForm] = useState<TransportForm | null>(null);
    const editing = !!form && transports.some((transport) => transport.id === form.transport.id);
    const error = form ? transportFormError(form) : null;

    const openEdit = (transport: TransportConfig) => setForm(transportFormFromConfig(transport));
    const selectType = (type: TransportTypeValue) => setForm((current) => current && switchTransportType(current, type));
    const save = () => {
        if (!form || error) return;
        const next = transportConfigFromForm(form);
        setTransports((list) => list.some((transport) => transport.id === next.id)
            ? list.map((transport) => transport.id === next.id ? next : transport)
            : [...list, next]);
        setForm(null);
    };

    return (
        <div className="flex flex-col gap-5">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Transports</h2>
                <p className="text-sm text-gray-700 italic">Configure how agents built from this profile call back.</p>
            </div>

            <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between gap-2">
                    <Heading size="sm">Configured transports ({transports.length})</Heading>
                    {!form && <Button buttonVariant="ghost" buttonStyle={{ color: "purple", size: "sm" }}
                        leftIcon={<Plus className="h-4 w-4" />} onClick={() => setForm(newTransportForm())}>Add transport</Button>}
                </div>

                <TransportList transports={transports} editingId={form?.transport.id} onEdit={openEdit}
                    onRemove={(transport) => {
                        setTransports((list) => list.filter((item) => item.id !== transport.id));
                        if (form?.transport.id === transport.id) setForm(null);
                    }} />
            </div>

            {form && <TransportEditor form={form} editing={editing} error={error} setForm={setForm}
                onTypeChange={selectType} onCancel={() => setForm(null)} onSave={save} />}

            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} aria-label="back button">Back</Button>
                <Button buttonStyle={{ color: "purple", size: "md" }} disabled={transports.length === 0 || !!form}
                    aria-label="continue transports step" onClick={onNext}>Continue</Button>
            </div>
        </div>
    );
};
