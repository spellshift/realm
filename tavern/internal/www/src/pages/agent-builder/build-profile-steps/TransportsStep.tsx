import { Dispatch, SetStateAction, useState } from "react";
import { Heading } from "@chakra-ui/react";
import { Trash2, Plus } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import SingleDropdownSelector from "../../../components/tavern-base-ui/SingleDropdownSelector";
import Modal from "../../../components/tavern-base-ui/Modal";
import { SupportedTransports } from "../../../utils/enums";
import { TRANSPORT_TYPES, TransportConfig } from "../types";

type TransportsStepProps = {
    transports: TransportConfig[];
    setTransports: Dispatch<SetStateAction<TransportConfig[]>>;
    onBack: () => void;
    onNext: () => void;
};

export const TransportsStep = ({ transports, setTransports, onBack, onNext }: TransportsStepProps) => {
    const [modalOpen, setModalOpen] = useState(false);
    const [draft, setDraft] = useState<TransportConfig | null>(null);

    const openAdd = () => {
        setDraft({ id: `t${Date.now()}`, type: SupportedTransports.HTTP1, uri: "", interval: 30 });
        setModalOpen(true);
    };
    const openEdit = (transport: TransportConfig) => {
        setDraft({ ...transport });
        setModalOpen(true);
    };
    const draftValid = !!draft && !!draft.uri.trim() && Number.isInteger(draft.interval) && draft.interval > 0;
    const save = () => {
        if (!draft || !draftValid) return;
        const next = { ...draft, uri: draft.uri.trim() };
        setTransports((list) => list.some((transport) => transport.id === next.id)
            ? list.map((transport) => transport.id === next.id ? next : transport)
            : [...list, next]);
        setModalOpen(false);
    };
    const remove = (id: string) => setTransports((list) => list.filter((x) => x.id !== id));

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Transports</h2>
                <p className="text-sm text-gray-700 italic">Configure how agents built from this profile call back.</p>
            </div>
            <div className="flex flex-col gap-3">
                <Heading size="sm">Transports ({transports.length})</Heading>
                <div className="flex flex-col gap-3">
                    {transports.map((t) => (
                        <div key={t.id} className="flex items-center gap-3 min-w-0">
                            <div className="flex flex-col min-w-0 flex-1">
                                <span className="text-sm font-medium text-gray-900">
                                    {TRANSPORT_TYPES.find((type) => type.value === t.type)?.label} · Every {t.interval}s
                                </span>
                                <span className="text-xs text-gray-500 break-all">{t.uri}</span>
                            </div>
                            <Button buttonVariant="ghost" buttonStyle={{ color: "purple", size: "sm" }} onClick={() => openEdit(t)}>
                                Edit
                            </Button>
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "square_sm" }}
                                onClick={() => remove(t.id)}
                                disabled={transports.length === 1}
                                aria-label="remove transport"
                            >
                                <Trash2 className="w-4 h-4" />
                            </Button>
                        </div>
                    ))}
                </div>
                <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "purple", size: "sm" }}
                    leftIcon={<Plus className="w-4 h-4" />}
                    onClick={openAdd}
                    className="w-fit"
                >
                    Add transport
                </Button>
            </div>
            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} aria-label="back button">Back</Button>
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    disabled={transports.length === 0}
                    aria-label="continue transports step"
                    onClick={onNext}
                >
                    Continue
                </Button>
            </div>

            <Modal isOpen={modalOpen} setOpen={setModalOpen} size="sm">
                <div className="flex flex-col gap-6">
                    <div>
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">
                            {transports.some((transport) => transport.id === draft?.id) ? "Edit transport" : "Add transport"}
                        </h3>
                        <p className="mt-1 text-sm text-gray-500">Configure the transport used by this build profile.</p>
                    </div>
                    {draft && (
                        <div className="flex flex-col gap-4">
                            <div>
                                <Heading size="sm" className="mb-1">Type</Heading>
                                <SingleDropdownSelector
                                    key={`${draft.id}-${draft.type}`}
                                    label="Type"
                                    options={TRANSPORT_TYPES.map((type) => ({ label: type.label, value: type.value }))}
                                    defaultValue={TRANSPORT_TYPES.find((type) => type.value === draft.type)}
                                    setSelectedOption={(option: { value: TransportConfig["type"] }) =>
                                        setDraft((current) => current && { ...current, type: option.value })}
                                />
                            </div>
                            <FormTextField
                                htmlFor="transportUri"
                                label="URI"
                                placeholder="http://127.0.0.1:8000"
                                value={draft.uri}
                                onChange={(event) => setDraft((current) => current && { ...current, uri: event.target.value })}
                            />
                            <FormTextField
                                htmlFor="transportInterval"
                                label="Interval (s)"
                                type="number"
                                min={1}
                                value={draft.interval}
                                onChange={(event) => setDraft((current) => current && { ...current, interval: Number(event.target.value) })}
                            />
                        </div>
                    )}
                    <div className="flex flex-row gap-2">
                        <Button buttonVariant="ghost" onClick={() => setModalOpen(false)}>Cancel</Button>
                        <Button onClick={save} disabled={!draftValid}>Save transport</Button>
                    </div>
                </div>
            </Modal>
        </div>
    );
};
