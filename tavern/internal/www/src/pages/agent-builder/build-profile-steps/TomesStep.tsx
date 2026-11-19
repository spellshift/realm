import { Dispatch, SetStateAction, useState } from "react";
import { useFormik } from "formik";
import { Heading } from "@chakra-ui/react";
import { Pencil, Trash2, Plus, TriangleAlert } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { ConfigureParams } from "../../../components/create-quest-modal/tome-selection/ConfigureParams";
import { ModalQuestFormValues } from "../../../components/create-quest-modal/types";
import { modalQuestSchema } from "../../../components/create-quest-modal/validation";
import { TomeTableWrapper } from "../../../components/create-quest-modal/tome-selection/TomeTableWrapper";
import { useTomeIds } from "../../../components/create-quest-modal/tome-selection/useTomeIds";
import { TomeNode } from "../../../utils/interfacesQuery";
import { NO_INSTALL_WARNING, Tome, TomeRef, TomeTacticKey } from "../types";
import { buildParamFields, ParamNotice, serializeParams } from "./tomeParams";

const paramsSchema = modalQuestSchema.pick(["params"]);
const emptyValues: ModalQuestFormValues = { name: "", tomeId: null, params: [], beacons: [] };

const noFilters = { tomeMultiSearch: "", tomeFields: [] };

type TomePickerProps = { selectedTomeId?: string | null; onSelectTome: (tome: TomeNode) => void };

// TomeTableWrapper ignores useTomeIds' error, so a failed fetch would read as "No tomes available".
// Probe the unfiltered list (same variables as the wrapper's initial query, so Apollo dedupes it) and show a retry instead.
const TomePicker = ({ selectedTomeId, onSelectTome }: TomePickerProps) => {
    const { tomeIds, error, refetch } = useTomeIds([], "");
    if (error && tomeIds.length === 0) {
        return (
            <div role="alert" className="flex items-center justify-between gap-2 rounded border border-red-200 bg-red-50 p-3">
                <p className="text-sm text-red-700">Failed to load tomes. Check your connection and try again.</p>
                <Button buttonVariant="ghost" onClick={() => refetch()}>Retry</Button>
            </div>
        );
    }
    return <TomeTableWrapper selectable selectedTomeId={selectedTomeId} onSelectTome={onSelectTome} showFilters initialFilters={noFilters} />;
};

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
    const [droppedNotice, setDroppedNotice] = useState<ParamNotice>(null);
    // Stored params kept verbatim when the tome definition can't be read (tome removed or bad paramDefs).
    const [keepStored, setKeepStored] = useState<string | null>(null);
    const panelOpen = adding || editingId !== null;

    const formik = useFormik<ModalQuestFormValues>({
        initialValues: emptyValues,
        validationSchema: paramsSchema,
        onSubmit: () => undefined,
    });

    const loadFields = (tome: Tome, stored?: string, missing = false) => {
        const { fields, notice, unavailable } = buildParamFields(tome.paramDefs, stored);
        formik.resetForm({ values: { ...emptyValues, tomeId: tome.id, params: fields } });
        setDroppedNotice(notice);
        setKeepStored(stored && (missing || unavailable) ? stored : null);
    };

    const selectTome = (tome: TomeNode) => {
        if (tomeToAdd?.id === tome.id) return;
        const selected: Tome = { id: tome.id, name: tome.name, description: tome.description ?? "", tactic: tome.tactic as TomeTacticKey, paramDefs: tome.paramDefs };
        setTomeToAdd(selected);
        loadFields(selected);
    };

    const resetPanelState = () => {
        formik.resetForm({ values: emptyValues });
        setDroppedNotice(null);
        setKeepStored(null);
    };

    const openAdd = () => {
        setEditingId(null);
        setTomeToAdd(null);
        resetPanelState();
        setAdding(true);
    };

    const openEdit = (ref: TomeRef) => {
        const found = catalog.find((tome) => tome.id === ref.tomeId);
        const catalogTome = found ?? { id: ref.tomeId, name: ref.tomeName, description: "", tactic: ref.tactic, paramDefs: null };
        setTomeToAdd(catalogTome);
        loadFields(catalogTome, ref.params, !found);
        setAdding(false);
        setEditingId(ref.id);
    };

    const closePanel = () => {
        setAdding(false);
        setEditingId(null);
        setTomeToAdd(null);
        resetPanelState();
    };

    const confirmSave = async () => {
        if (!tomeToAdd) return;
        formik.setTouched({ params: formik.values.params.map(() => ({ value: true })) as never }, false);
        const errors = await formik.validateForm();
        if (errors.params) {
            onError(null);
            return;
        }
        onError(null);
        const params = keepStored ?? serializeParams(formik.values.params);
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
                                    className="rounded p-1 text-gray-400 hover:text-gray-600 disabled:cursor-not-allowed disabled:opacity-40"
                                    aria-label={`Edit ${t.tomeName} tome`}
                                    disabled={panelOpen}
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
                            : <TomePicker selectedTomeId={tomeToAdd?.id} onSelectTome={selectTome} />}
                        {tomeToAdd && keepStored === null && <ConfigureParams formik={formik} />}
                        {tomeToAdd && keepStored !== null && (
                            <p className="text-xs text-amber-700">This tome's parameter definitions are unavailable, so its stored parameters will be kept unchanged. Select a tome above to replace them.</p>
                        )}
                        {tomeToAdd && keepStored === null && droppedNotice === "dropped" && (
                            <p className="text-xs text-amber-700">Some stored parameters no longer match this tome and will be removed when you save.</p>
                        )}
                        {tomeToAdd && keepStored === null && droppedNotice === "invalid" && (
                            <p className="text-xs text-amber-700">The stored parameters could not be read, so the fields were reset to defaults.</p>
                        )}
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
