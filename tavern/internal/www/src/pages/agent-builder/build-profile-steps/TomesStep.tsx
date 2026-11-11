import { Dispatch, SetStateAction, useState } from "react";
import { Heading } from "@chakra-ui/react";
import { Trash2, Plus, TriangleAlert } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import { FormRadioGroup } from "../../../components/tavern-base-ui/FormRadioGroup";
import { safelyJsonParse } from "../../../utils/utils";
import { NO_INSTALL_WARNING, Tome, TomeRef } from "../types";

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
    const [tomeToAdd, setTomeToAdd] = useState<Tome | null>(null);
    const [params, setParams] = useState("{}");

    const confirmAdd = () => {
        if (!tomeToAdd) return;
        try {
            const parsed = JSON.parse(params);
            if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") throw new Error("Tome parameters must be a JSON object.");
        } catch (error) {
            onError(error instanceof Error ? error.message : String(error));
            return;
        }
        onError(null);
        setTomes((list) => [
            ...list,
            { id: `tr${list.length + 1}-${Date.now()}`, tomeId: tomeToAdd.id, tomeName: tomeToAdd.name, paramSummary: params === "{}" ? "No params" : params, params, tactic: tomeToAdd.tactic },
        ]);
        setAdding(false);
    };

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Bundled tomes</h2>
                <p className="text-sm text-gray-700 italic">Tomes that run automatically when the agent starts.</p>
            </div>
            <div className="flex flex-col gap-3">
                <Heading size="sm">Bundled tomes ({tomes.length})</Heading>
                {!hasPersistenceTome && (
                    <div className="flex flex-row gap-2">
                        <TriangleAlert className="h-5 w-5 text-amber-400 flex-shrink-0" aria-hidden="true" />
                        <p className="text-sm text-amber-700">{NO_INSTALL_WARNING}</p>
                    </div>
                )}
                {tomes.length === 0 && !adding && (
                    <p className="text-sm text-gray-500">No tomes bundled yet.</p>
                )}
                <div className="flex flex-col gap-3">
                    {tomes.map((t) => (
                        <div key={t.id} className="flex flex-row items-center justify-between">
                            <div className="flex flex-col">
                                <span className="text-sm font-medium text-gray-900">{t.tomeName}</span>
                                <span className="text-xs text-gray-500">{t.paramSummary}</span>
                            </div>
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "square_sm" }}
                                onClick={() => setTomes((list) => list.filter((x) => x.id !== t.id))}
                                aria-label="remove tome"
                            >
                                <Trash2 className="w-4 h-4" />
                            </Button>
                        </div>
                    ))}
                </div>

                {adding ? (
                    <div className="flex flex-col gap-3 pt-1">
                        <FormRadioGroup
                            label="Choose a tome"
                            data={catalog}
                            selected={tomeToAdd}
                            setSelected={(tome: Tome) => {
                                setTomeToAdd(tome);
                                const { params: definitions } = safelyJsonParse(tome.paramDefs || "[]");
                                setParams(JSON.stringify(Object.fromEntries(definitions.map((param: { name: string; value?: string }) => [param.name, param.value || ""])), null, 2));
                            }}
                        />
                        {catalog.length === 0 && <p className="text-sm text-gray-500">No tomes available. Import a tome before adding it to this profile.</p>}
                        {tomeToAdd && <FormTextField htmlFor="tomeParams" label="Parameters (JSON object)" value={params} onChange={(event) => setParams(event.target.value)} />}
                        <div className="flex flex-row gap-2">
                            <Button buttonVariant="ghost" buttonStyle={{ size: "sm" }} onClick={() => setAdding(false)}>
                                Cancel
                            </Button>
                            <Button buttonStyle={{ color: "purple", size: "sm" }} onClick={confirmAdd} disabled={!tomeToAdd}>
                                Add to profile
                            </Button>
                        </div>
                    </div>
                ) : (
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "purple", size: "sm" }}
                        leftIcon={<Plus className="w-4 h-4" />}
                        onClick={() => setAdding(true)}
                        className="w-fit"
                    >
                        Add tome
                    </Button>
                )}
            </div>
            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} aria-label="back button">Back</Button>
                <Button buttonStyle={{ color: "purple", size: "md" }} aria-label="continue tomes step" onClick={onNext}>
                    Continue
                </Button>
            </div>
        </div>
    );
};
