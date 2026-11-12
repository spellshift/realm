import { Dispatch, SetStateAction, useState } from "react";
import { ChevronDown, ChevronUp, TriangleAlert } from "lucide-react";
import { Textarea } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { NO_INSTALL_WARNING, TRANSPORT_TYPES, TomeRef, TransportConfig } from "../types";
import { advancedScriptFields, textareaClass } from "./constants";
import { UNIQUE_FIELD_INDEX } from "./validation";

type ReviewStepProps = {
    name: string;
    description: string;
    buildImage: string;
    transports: TransportConfig[];
    tomes: TomeRef[];
    hasPersistenceTome: boolean;
    scripts: string[];
    uniqueError: string | null;
    setScripts: Dispatch<SetStateAction<string[]>>;
    isEditing: boolean;
    saving: boolean;
    canSave: boolean;
    onBack: () => void;
    onSave: () => void;
};

export const ReviewStep = ({
    name,
    description,
    buildImage,
    transports,
    tomes,
    hasPersistenceTome,
    scripts,
    uniqueError,
    setScripts,
    isEditing,
    saving,
    canSave,
    onBack,
    onSave,
}: ReviewStepProps) => {
    // Open Advanced up front when a value in it needs fixing, so the error is visible.
    const [showAdvanced, setShowAdvanced] = useState(!!uniqueError);

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Review &amp; save</h2>
                <p className="text-sm text-gray-700 italic">Confirm the details below, then save the profile.</p>
            </div>

            <div className="rounded-md border border-gray-200 divide-y divide-gray-200 text-sm">
                <div className="p-3 flex justify-between">
                    <span className="text-gray-500">Name</span>
                    <span className="font-medium text-gray-900">{name}</span>
                </div>
                {description && (
                    <div className="p-3 flex flex-col gap-1">
                        <span className="text-gray-500">Description</span>
                        <span className="text-gray-900">{description}</span>
                    </div>
                )}
                <div className="p-3 flex justify-between">
                    <span className="text-gray-500">Build image</span>
                    <code className="text-xs font-mono text-gray-900">{buildImage}</code>
                </div>
                <div className="p-3 flex flex-col gap-2">
                    <div className="flex justify-between items-center">
                        <span className="text-gray-500">Transports</span>
                        <span className="text-xs text-gray-500">{transports.length} configured</span>
                    </div>
                    <div className="flex flex-col gap-2">
                        {transports.map((t) => (
                            <div key={t.id} className="rounded border border-gray-200 bg-gray-50/50 p-2.5 text-xs flex flex-col gap-1">
                                <div className="flex items-center justify-between gap-2">
                                    <div className="flex items-center gap-2 min-w-0">
                                        <span className="font-semibold text-gray-900">
                                            {TRANSPORT_TYPES.find((type) => type.value === t.type)?.label}
                                        </span>
                                        <span className="text-gray-500 font-mono break-all">{t.uri}</span>
                                    </div>
                                    <span className="text-gray-500 flex-shrink-0 ml-2">{t.interval}s interval</span>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
                <div className="p-3 flex justify-between">
                    <span className="text-gray-500">Bundled tomes</span>
                    <div className="flex flex-col gap-1 items-end">
                        {tomes.length === 0 ? (
                            <span className="text-gray-400">None</span>
                        ) : (
                            tomes.map((t) => (
                                <span key={t.id} className="font-medium text-gray-900">{t.tomeName}</span>
                            ))
                        )}
                    </div>
                </div>
            </div>
            {!hasPersistenceTome && (
                <div className="flex flex-row gap-2">
                    <TriangleAlert className="h-5 w-5 text-amber-400 flex-shrink-0" aria-hidden="true" />
                    <p className="text-sm text-amber-700">{NO_INSTALL_WARNING}</p>
                </div>
            )}

            <div className="flex flex-col gap-2 pt-2">
                <button
                    type="button"
                    className="flex flex-row items-center gap-1 text-sm font-medium text-purple-700 text-left w-fit"
                    onClick={() => setShowAdvanced((v) => !v)}
                >
                    Advanced
                    {showAdvanced ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                </button>
                <p className="text-xs text-gray-500 -mt-1">
                    Go-template build/setup scripts and artifact path. Defaults work for
                    standard imix builds -- only change these for custom build pipelines.
                </p>
                {showAdvanced && (
                    <div className="flex flex-col gap-4 mt-2">
                        {advancedScriptFields.map((field, index) => (
                            <div key={field.label} className="flex flex-col gap-1">
                                <label className="block text-sm font-medium text-gray-900">{field.label}</label>
                                <Textarea className={textareaClass} rows={field.rows} value={scripts[index]} onChange={(event) => setScripts((current) => current.map((value, i) => i === index ? event.target.value : value))} />
                                {field.description && <p className="text-xs text-gray-500">{field.description}</p>}
                                {index === UNIQUE_FIELD_INDEX && uniqueError && <p role="alert" className="text-xs text-red-600">{uniqueError}</p>}
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} disabled={saving} aria-label="back button">Back</Button>
                <Button onClick={onSave} isLoading={saving} disabled={!canSave} aria-label="submit profile">
                    {isEditing ? "Save changes" : "Save profile"}
                </Button>
            </div>
        </div>
    );
};
