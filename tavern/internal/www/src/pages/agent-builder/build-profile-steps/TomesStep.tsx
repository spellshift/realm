import { Dispatch, SetStateAction, useState } from "react";
import { Heading } from "@chakra-ui/react";
import { Pencil, Trash2, Plus, TriangleAlert } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import { TomeTable } from "../../../components/create-quest-modal/tome-selection/TomeTable";
import { TomeNode } from "../../../utils/interfacesQuery";
import { safelyJsonParse } from "../../../utils/utils";
import { NO_INSTALL_WARNING, Tome, TomeRef, TomeTacticKey } from "../types";

type TomesStepProps = {
    catalog: Tome[];
    tomes: TomeRef[];
    setTomes: Dispatch<SetStateAction<TomeRef[]>>;
    hasPersistenceTome: boolean;
    onError: (message: string | null) => void;
    onBack: () => void;
    onNext: () => void;
};

export const TomesStep = ({ catalog, tomes, setTomes, hasPersistenceTome, onError, onBack, onNext }: TomesStepProps) => {
    const [adding, setAdding] = useState(false);
    const [editingId, setEditingId] = useState<string | null>(null);
    const [tomeToAdd, setTomeToAdd] = useState<Tome | null>(null);
    const [params, setParams] = useState("{}");
    const panelOpen = adding || editingId !== null;

    const selectTome = (tome: TomeNode) => {
        setTomeToAdd({ id: tome.id, name: tome.name, description: tome.description ?? "", tactic: tome.tactic as TomeTacticKey, paramDefs: tome.paramDefs });
        const { params: definitions } = safelyJsonParse(tome.paramDefs || "[]");
        setParams(JSON.stringify(Object.fromEntries(definitions.map((param: { name: string; value?: string }) => [param.name, param.value || ""])), null, 2));
    };

    const openAdd = () => {
        setEditingId(null);
        setTomeToAdd(null);
        setParams("{}");
        setAdding(true);
    };

    const openEdit = (ref: TomeRef) => {
        const catalogTome = catalog.find((tome) => tome.id === ref.tomeId);
        setTomeToAdd(catalogTome ?? { id: ref.tomeId, name: ref.tomeName, description: "", tactic: ref.tactic, paramDefs: null });
        setParams(ref.params);
        setAdding(false);
        setEditingId(ref.id);
    };

    const closePanel = () => {
        setAdding(false);
        setEditingId(null);
        setTomeToAdd(null);
    };

    const confirmSave = () => {
        if (!tomeToAdd) return;
        try {
            const parsed = JSON.parse(params);
            if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") throw new Error("Tome parameters must be a JSON object.");
        } catch (error) {
            onError(error instanceof Error ? error.message : String(error));
            return;
        }
        onError(null);
        const paramSummary = params === "{}" ? "No params" : params;
        if (editingId) {
            setTomes((list) => list.map((t) => t.id === editingId
                ? { ...t, tomeId: tomeToAdd.id, tomeName: tomeToAdd.name, paramSummary, params, tactic: tomeToAdd.tactic }
                : t));
        } else {
            setTomes((list) => [
                ...list,
                { id: `tr${list.length + 1}-${Date.now()}`, tomeId: tomeToAdd.id, tomeName: tomeToAdd.name, paramSummary, params, tactic: tomeToAdd.tactic },
            ]);
        }
        closePanel();
    };

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Bundled tomes</h2>
                <p className="text-sm text-gray-700 italic">Tomes that run automatically when the agent starts.</p>
            </div>
            <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between gap-2">
                    <Heading size="sm">Bundled tomes ({tomes.length})</Heading>
                    {!panelOpen && <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "purple", size: "sm" }}
                        leftIcon={<Plus className="w-4 h-4" />}
                        onClick={openAdd}
                    >
                        Add tome
                    </Button>}
                </div>
                <div className="flex flex-col gap-3">
                    {tomes.map((t) => (
                        <div key={t.id} className="flex flex-row items-center justify-between">
                            <div className={`flex flex-col ${editingId === t.id ? "border-l-[3px] border-purple-600 pl-[9px]" : ""}`}>
                                <span className="text-sm font-medium text-gray-900">{t.tomeName}</span>
                                <span className="text-xs text-gray-500">{t.paramSummary}</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <button
                                    type="button"
                                    className="rounded p-1 text-gray-400 hover:text-gray-600"
                                    aria-label={`Edit ${t.tomeName} tome`}
                                    onClick={() => openEdit(t)}
                                >
                                    <Pencil className="w-4 h-4" />
                                </button>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "red", size: "square_sm" }}
                                    onClick={() => {
                                        setTomes((list) => list.filter((x) => x.id !== t.id));
                                        if (editingId === t.id) closePanel();
                                    }}
                                    aria-label="remove tome"
                                >
                                    <Trash2 className="w-4 h-4" />
                                </Button>
                            </div>
                        </div>
                    ))}
                </div>

                {panelOpen && (
                    <section className="flex flex-col gap-4 rounded-lg border border-gray-200 bg-white p-5 shadow-sm" aria-label={editingId ? "Edit tome" : "Add tome"}>
                        <h3 className="text-base font-semibold text-gray-900">{editingId ? "Edit tome" : "Add tome"}</h3>
                        {catalog.length === 0
                            ? <p className="text-sm text-gray-500">No tomes available. Import a tome before adding it to this profile.</p>
                            : <TomeTable tomeIds={catalog.map((tome) => tome.id)} selectable selectedTomeId={tomeToAdd?.id} onSelectTome={selectTome} />}
                        {tomeToAdd && <FormTextField htmlFor="tomeParams" label="Parameters" value={params} onChange={(event) => setParams(event.target.value)} />}
                        <div className="flex flex-wrap items-center justify-end gap-2">
                            <Button buttonVariant="ghost" onClick={closePanel}>
                                Cancel
                            </Button>
                            <Button onClick={confirmSave} disabled={!tomeToAdd}>
                                {editingId ? "Save tome" : "Add to profile"}
                            </Button>
                        </div>
                    </section>
                )}
            </div>
            {tomes.length === 0 ? (
                <p className="text-sm text-gray-500">No tomes bundled yet.</p>
            ) : !hasPersistenceTome && (
                <div className="flex flex-row gap-2">
                    <TriangleAlert className="h-5 w-5 text-amber-400 flex-shrink-0" aria-hidden="true" />
                    <p className="text-sm text-amber-700">{NO_INSTALL_WARNING}</p>
                </div>
            )}
            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} aria-label="back button">Back</Button>
                <Button buttonStyle={{ color: "purple", size: "md" }} aria-label="continue tomes step" onClick={onNext}>
                    Continue
                </Button>
            </div>
        </div>
    );
};
