import React, { Fragment, useEffect, useMemo, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { BookOpenIcon, CheckIcon, XMarkIcon } from "@heroicons/react/24/outline";
import { useMutation } from "@apollo/client";
import {
    AlertCircle,
    AlertTriangle,
    CheckCircle2,
    ChevronDown,
    Loader2,
    Plus,
    Trash2,
} from "lucide-react";
import { StepStatus, TomeTactic } from "../../../utils/enums";
import { CREATE_TOME_MUTATION, GET_SHELL_TOMES } from "../graphql";
import { ShellTomeItem } from "../hooks/useShellData";
import { analyzeEldritchCode } from "../utils/eldritchLsp";
import {
    formatParamLabel,
    parseComposerScriptForTome,
    serializeTomeParamDefs,
    TomeParamDefInput,
} from "../utils/tomeComposerUtils";

export interface SaveTomeModalProps {
    isOpen: boolean;
    onClose: () => void;
    composerCode: string;
    existingTomes?: ShellTomeItem[];
    defaultAuthor?: string;
    onTomeCreated?: (createdTome: ShellTomeItem) => void;
}

interface VerticalStepItem {
    name: string;
    description: string;
    step: number;
}

const STEPS: VerticalStepItem[] = [
    { name: "Configure details", description: "Step 1", step: 0 },
    { name: "Define parameters", description: "Step 2", step: 1 },
    { name: "Confirm tome", description: "Step 3", step: 2 },
];

const TACTIC_OPTIONS = Object.entries(TomeTactic).map(([key, label]) => ({
    value: key,
    label,
}));

const PARAM_TYPE_OPTIONS = [
    { value: "string", label: "string" },
    { value: "int32", label: "int32" },
    { value: "bool", label: "bool" },
];

const DARK_INPUT_CLASS =
    "w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-xs text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500";

const getStepStatus = (currStep: number, targetStep: number): StepStatus => {
    if (currStep > targetStep) {
        return StepStatus.Complete;
    }
    if (currStep === targetStep) {
        return StepStatus.Current;
    }
    return StepStatus.Upcoming;
};

const SaveTomeVerticalSteps: React.FC<{ currStep: number; steps: VerticalStepItem[] }> = ({
    currStep,
    steps,
}) => (
    <nav aria-label="Progress">
        <ol className="overflow-hidden">
            {steps.map((s, stepIdx) => {
                const status = getStepStatus(currStep, s.step);
                const isLast = stepIdx === steps.length - 1;

                if (status === StepStatus.Complete) {
                    return (
                        <li key={s.name} className={`${!isLast ? "pb-10" : ""} relative`}>
                            {!isLast && (
                                <div
                                    className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-purple-600"
                                    aria-hidden="true"
                                />
                            )}
                            <div className="group relative flex items-start">
                                <span className="flex h-9 items-center">
                                    <span className="relative z-10 flex h-8 w-8 items-center justify-center rounded-full bg-purple-600">
                                        <CheckIcon
                                            className="h-4 w-4 text-white stroke-[2.5]"
                                            aria-hidden="true"
                                        />
                                    </span>
                                </span>
                                <span className="ml-3.5 flex min-w-0 flex-col">
                                    <span className="text-xs font-medium text-gray-200">
                                        {s.name}
                                    </span>
                                    <span className="text-xs text-gray-400">{s.description}</span>
                                </span>
                            </div>
                        </li>
                    );
                }

                if (status === StepStatus.Current) {
                    return (
                        <li key={s.name} className={`${!isLast ? "pb-10" : ""} relative`}>
                            {!isLast && (
                                <div
                                    className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-[#343b4d]"
                                    aria-hidden="true"
                                />
                            )}
                            <div className="group relative flex items-start" aria-current="step">
                                <span className="flex h-9 items-center" aria-hidden="true">
                                    <span className="relative z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 border-purple-500 bg-[#12151c]">
                                        <span className="h-2.5 w-2.5 rounded-full bg-purple-400" />
                                    </span>
                                </span>
                                <span className="ml-3.5 flex min-w-0 flex-col">
                                    <span className="text-xs font-semibold text-purple-400">
                                        {s.name}
                                    </span>
                                    <span className="text-xs text-gray-400">{s.description}</span>
                                </span>
                            </div>
                        </li>
                    );
                }

                return (
                    <li key={s.name} className={`${!isLast ? "pb-10" : ""} relative`}>
                        {!isLast && (
                            <div
                                className="absolute top-4 left-4 -ml-px mt-0.5 h-full w-0.5 bg-[#343b4d]"
                                aria-hidden="true"
                            />
                        )}
                        <div className="group relative flex items-start">
                            <span className="flex h-9 items-center" aria-hidden="true">
                                <span className="relative z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 border-[#343b4d] bg-[#12151c]">
                                    <span className="h-2.5 w-2.5 rounded-full bg-transparent" />
                                </span>
                            </span>
                            <span className="ml-3.5 flex min-w-0 flex-col">
                                <span className="text-xs font-medium text-gray-500">{s.name}</span>
                                <span className="text-xs text-gray-500">{s.description}</span>
                            </span>
                        </div>
                    </li>
                );
            })}
        </ol>
    </nav>
);

interface StepTomeDetailsProps {
    name: string;
    onNameChange: (next: string) => void;
    description: string;
    onDescriptionChange: (next: string) => void;
    tactic: string;
    onTacticChange: (next: string) => void;
    author: string;
    onAuthorChange: (next: string) => void;
    isDuplicateName: boolean;
    canContinue: boolean;
    onCancel: () => void;
    onContinue: () => void;
}

const StepTomeDetails: React.FC<StepTomeDetailsProps> = ({
    name,
    onNameChange,
    description,
    onDescriptionChange,
    tactic,
    onTacticChange,
    author,
    onAuthorChange,
    isDuplicateName,
    canContinue,
    onCancel,
    onContinue,
}) => (
    <div data-testid="save-tome-details-step" className="flex flex-col gap-5">
        <div className="flex flex-col gap-1">
            <h2 className="text-lg font-semibold text-white">Configure tome details</h2>
            <p className="text-xs text-gray-400 italic">
                Provide a recognizable name, description, and tactic for this tome
            </p>
        </div>

        <div className="flex flex-col gap-3.5">
            <div>
                <label
                    htmlFor="save-tome-name"
                    className="block text-xs font-medium text-gray-300 mb-1.5"
                >
                    Tome Name <span className="text-red-400">*</span>
                </label>
                <input
                    id="save-tome-name"
                    type="text"
                    value={name}
                    onChange={(e) => onNameChange(e.target.value)}
                    placeholder="e.g. Find SUID Binaries"
                    autoFocus
                    className={`${DARK_INPUT_CLASS} ${
                        isDuplicateName ? "border-red-500 focus:border-red-400 focus:ring-red-400" : ""
                    }`}
                />
                {isDuplicateName && (
                    <p
                        data-testid="save-tome-duplicate-error"
                        className="mt-1 text-[11px] text-red-400"
                    >
                        A tome with this name already exists. Please choose a unique name.
                    </p>
                )}
            </div>

            <div>
                <label
                    htmlFor="save-tome-description"
                    className="block text-xs font-medium text-gray-300 mb-1.5"
                >
                    Description <span className="text-red-400">*</span>
                </label>
                <textarea
                    id="save-tome-description"
                    rows={3}
                    value={description}
                    onChange={(e) => onDescriptionChange(e.target.value)}
                    placeholder="Describe what this tome does..."
                    className={`${DARK_INPUT_CLASS} resize-none`}
                />
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                    <label
                        htmlFor="save-tome-tactic"
                        className="block text-xs font-medium text-gray-300 mb-1.5"
                    >
                        Tactic
                    </label>
                    <div className="relative">
                        <select
                            id="save-tome-tactic"
                            value={tactic}
                            onChange={(e) => onTacticChange(e.target.value)}
                            className={`${DARK_INPUT_CLASS} appearance-none pr-8`}
                        >
                            {TACTIC_OPTIONS.map((opt) => (
                                <option key={opt.value} value={opt.value}>
                                    {opt.label}
                                </option>
                            ))}
                        </select>
                        <ChevronDown
                            className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400"
                            aria-hidden="true"
                        />
                    </div>
                </div>

                <div>
                    <label
                        htmlFor="save-tome-author"
                        className="block text-xs font-medium text-gray-300 mb-1.5"
                    >
                        Author <span className="text-red-400">*</span>
                    </label>
                    <input
                        id="save-tome-author"
                        type="text"
                        value={author}
                        onChange={(e) => onAuthorChange(e.target.value)}
                        placeholder="operator"
                        className={DARK_INPUT_CLASS}
                    />
                </div>
            </div>
        </div>

        <div className="flex flex-row gap-2 pt-1">
            <button
                type="button"
                onClick={onCancel}
                aria-label="close modal"
                className="text-xs font-semibold py-2 px-3.5 rounded-md bg-[#252b3b] hover:bg-[#2f374b] text-gray-200 border border-[#363f54] transition-colors"
            >
                Cancel
            </button>
            <button
                type="button"
                data-testid="save-tome-next-button"
                onClick={onContinue}
                disabled={!canContinue}
                aria-label="continue details step"
                className="text-xs font-semibold py-2 px-4 rounded-md bg-purple-600 text-white hover:bg-purple-500 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
                Continue
            </button>
        </div>
    </div>
);

interface StepTomeParametersProps {
    params: TomeParamDefInput[];
    onAddParam: () => void;
    onUpdateParam: (idx: number, field: keyof TomeParamDefInput, value: string) => void;
    onRemoveParam: (idx: number) => void;
    canContinue: boolean;
    onBack: () => void;
    onContinue: () => void;
}

const StepTomeParameters: React.FC<StepTomeParametersProps> = ({
    params,
    onAddParam,
    onUpdateParam,
    onRemoveParam,
    canContinue,
    onBack,
    onContinue,
}) => (
    <div data-testid="save-tome-params-step" className="flex flex-col gap-5">
        <div className="flex flex-row items-start justify-between gap-3">
            <div className="flex flex-col gap-1">
                <h2 className="text-lg font-semibold text-white">Define parameters</h2>
                <p className="text-xs text-gray-400 italic">
                    Configure runtime parameters accessed via{" "}
                    <code className="font-mono not-italic text-gray-200 bg-[#12151c] px-1 py-0.5 rounded border border-[#2a3040]">
                        input_params["name"]
                    </code>
                </p>
            </div>

            <button
                type="button"
                data-testid="save-tome-add-param"
                onClick={onAddParam}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-[#252b3b] hover:bg-[#2f374b] border border-[#363f54] text-xs font-medium text-gray-200 shrink-0 transition-colors"
            >
                <Plus className="w-3.5 h-3.5" />
                Add parameter
            </button>
        </div>

        {params.length === 0 ? (
            <div
                data-testid="save-tome-no-params"
                className="rounded-md bg-[#141720] border border-[#2a3040] px-4 py-7 text-center text-xs text-gray-400"
            >
                No input parameters defined for this tome. Click{" "}
                <span className="text-gray-200 font-medium">Add parameter</span> if your script
                takes runtime inputs.
            </div>
        ) : (
            <div className="flex flex-col gap-2.5 max-h-64 overflow-y-auto pr-1">
                {params.map((param, idx) => (
                    <div
                        key={idx}
                        data-testid="save-tome-param-row"
                        className="rounded-md bg-[#141720] border border-[#2a3040] p-3 flex flex-col gap-2.5"
                    >
                        <div className="grid grid-cols-12 gap-2 items-center">
                            <div className="col-span-4">
                                <label className="block text-[10px] text-gray-400 mb-0.5">
                                    Key Name <span className="text-red-400">*</span>
                                </label>
                                <input
                                    type="text"
                                    aria-label="Parameter name"
                                    value={param.name}
                                    onChange={(e) => onUpdateParam(idx, "name", e.target.value)}
                                    placeholder="path"
                                    className="w-full px-2 py-1.5 bg-[#0f1117] border border-[#2f3647] rounded text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                                />
                            </div>

                            <div className="col-span-4">
                                <label className="block text-[10px] text-gray-400 mb-0.5">
                                    Label
                                </label>
                                <input
                                    type="text"
                                    aria-label="Parameter label"
                                    value={param.label}
                                    onChange={(e) => onUpdateParam(idx, "label", e.target.value)}
                                    placeholder={formatParamLabel(param.name) || "File path"}
                                    className="w-full px-2 py-1.5 bg-[#0f1117] border border-[#2f3647] rounded text-xs text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                                />
                            </div>

                            <div className="col-span-3">
                                <label className="block text-[10px] text-gray-400 mb-0.5">
                                    Type
                                </label>
                                <div className="relative">
                                    <select
                                        aria-label="Parameter type"
                                        value={param.type}
                                        onChange={(e) => onUpdateParam(idx, "type", e.target.value)}
                                        className="w-full pl-2 pr-7 py-1.5 bg-[#0f1117] border border-[#2f3647] rounded text-xs font-mono text-white appearance-none focus:outline-none focus:border-purple-500"
                                    >
                                        {PARAM_TYPE_OPTIONS.map((t) => (
                                            <option key={t.value} value={t.value}>
                                                {t.label}
                                            </option>
                                        ))}
                                    </select>
                                    <ChevronDown
                                        className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400"
                                        aria-hidden="true"
                                    />
                                </div>
                            </div>

                            <div className="col-span-1 flex justify-end pt-3.5">
                                <button
                                    type="button"
                                    aria-label="Remove parameter"
                                    onClick={() => onRemoveParam(idx)}
                                    className="p-1 rounded text-gray-400 hover:text-red-400 hover:bg-white/5 transition-colors"
                                >
                                    <Trash2 className="w-3.5 h-3.5" />
                                </button>
                            </div>
                        </div>

                        <div>
                            <label className="block text-[10px] text-gray-400 mb-0.5">
                                Placeholder / Default Hint
                            </label>
                            <input
                                type="text"
                                aria-label="Parameter placeholder"
                                value={param.placeholder}
                                onChange={(e) =>
                                    onUpdateParam(idx, "placeholder", e.target.value)
                                }
                                placeholder="e.g. /etc/passwd"
                                className="w-full px-2 py-1.5 bg-[#0f1117] border border-[#2f3647] rounded text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                            />
                        </div>
                    </div>
                ))}
            </div>
        )}

        <div className="flex flex-row gap-2 pt-1">
            <button
                type="button"
                data-testid="save-tome-back-button"
                onClick={onBack}
                aria-label="back to details"
                className="text-xs font-semibold py-2 px-3.5 rounded-md text-gray-300 hover:text-white hover:bg-[#2c3242] transition-colors"
            >
                Back
            </button>
            <button
                type="button"
                data-testid="save-tome-next-button"
                onClick={onContinue}
                disabled={!canContinue}
                aria-label="continue parameters step"
                className="text-xs font-semibold py-2 px-4 rounded-md bg-purple-600 text-white hover:bg-purple-500 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
                Continue
            </button>
        </div>
    </div>
);

const ReviewLspStatusBadge: React.FC<{ errorCount: number; warningCount: number }> = ({
    errorCount,
    warningCount,
}) => {
    if (errorCount > 0) {
        return (
            <span className="inline-flex items-center gap-1 text-red-400">
                <AlertCircle className="w-3.5 h-3.5" />
                {errorCount} {errorCount === 1 ? "error" : "errors"}
            </span>
        );
    }
    if (warningCount > 0) {
        return (
            <span className="inline-flex items-center gap-1 text-amber-300">
                <AlertTriangle className="w-3.5 h-3.5" />
                {warningCount} {warningCount === 1 ? "warning" : "warnings"}
            </span>
        );
    }
    return (
        <span className="inline-flex items-center gap-1 text-emerald-400">
            <CheckCircle2 className="w-3.5 h-3.5" />
            Syntax OK
        </span>
    );
};

interface StepTomeReviewProps {
    name: string;
    description: string;
    tactic: string;
    author: string;
    params: TomeParamDefInput[];
    eldritch: string;
    onEldritchChange: (next: string) => void;
    submitError: string | null;
    isSubmitting: boolean;
    canSubmit: boolean;
    onBack: () => void;
    onSubmit: () => void;
}

const StepTomeReview: React.FC<StepTomeReviewProps> = ({
    name,
    description,
    tactic,
    author,
    params,
    eldritch,
    onEldritchChange,
    submitError,
    isSubmitting,
    canSubmit,
    onBack,
    onSubmit,
}) => {
    const diagnostics = useMemo(() => analyzeEldritchCode(eldritch), [eldritch]);
    const errorCount = useMemo(
        () => diagnostics.filter((d) => d.severity === "error").length,
        [diagnostics]
    );
    const warningCount = useMemo(
        () => diagnostics.filter((d) => d.severity === "warning").length,
        [diagnostics]
    );

    const validParams = params.filter((p) => p.name.trim().length > 0);

    return (
        <div data-testid="save-tome-review-step" className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
                <h2 className="text-lg font-semibold text-white">Confirm tome details</h2>
                <p className="text-xs text-gray-400 italic">
                    Review metadata and Eldritch script before saving
                </p>
            </div>

            {submitError && (
                <div
                    data-testid="save-tome-submit-error"
                    className="rounded-md bg-red-500/10 border border-red-500/30 px-3 py-2 text-xs text-red-300 flex items-center gap-2"
                >
                    <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
                    <span>{submitError}</span>
                </div>
            )}

            {/* Metadata Summary */}
            <div className="rounded-md bg-[#141720] border border-[#2a3040] p-3 space-y-1.5">
                <div className="flex items-center justify-between gap-2">
                    <span className="text-xs font-semibold text-white truncate">{name}</span>
                    <div className="flex items-center gap-1.5 shrink-0">
                        <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/5 text-gray-300">
                            {TomeTactic[tactic as keyof typeof TomeTactic] || tactic}
                        </span>
                        <span className="text-[10px] text-gray-400">by {author}</span>
                    </div>
                </div>
                <p className="text-[11px] text-gray-400 line-clamp-2">{description}</p>
                {validParams.length > 0 && (
                    <div className="flex items-center flex-wrap gap-1 pt-1">
                        <span className="text-[10px] text-gray-500">Params:</span>
                        {validParams.map((p) => (
                            <span
                                key={p.name}
                                className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#1d222e] border border-[#2f3647] text-gray-300"
                            >
                                {p.name} ({p.type})
                            </span>
                        ))}
                    </div>
                )}
            </div>

            {/* Editable Eldritch Script with Live Diagnostics Status */}
            <div>
                <div className="flex items-center justify-between mb-1.5">
                    <label
                        htmlFor="save-tome-script"
                        className="block text-xs font-medium text-gray-300"
                    >
                        Eldritch Script <span className="text-red-400">*</span>
                    </label>
                    <span
                        data-testid="save-tome-lsp-status"
                        className="inline-flex items-center gap-1 text-[11px] font-mono"
                    >
                        <ReviewLspStatusBadge
                            errorCount={errorCount}
                            warningCount={warningCount}
                        />
                    </span>
                </div>
                <textarea
                    id="save-tome-script"
                    aria-label="Tome Eldritch Script"
                    rows={6}
                    value={eldritch}
                    onChange={(e) => onEldritchChange(e.target.value)}
                    spellCheck={false}
                    className="w-full px-3 py-2 bg-[#0f1118] border border-[#343b4d] rounded-md font-mono text-xs text-gray-100 placeholder-gray-500 focus:outline-none focus:border-purple-500"
                />
            </div>

            <div className="flex flex-row gap-2 pt-1">
                <button
                    type="button"
                    data-testid="save-tome-back-button"
                    onClick={onBack}
                    disabled={isSubmitting}
                    aria-label="back button"
                    className="text-xs font-semibold py-2 px-3.5 rounded-md text-gray-300 hover:text-white hover:bg-[#2c3242] disabled:opacity-40 transition-colors"
                >
                    Back
                </button>
                <button
                    type="submit"
                    data-testid="save-tome-submit-button"
                    onClick={(event) => {
                        event.preventDefault();
                        onSubmit();
                    }}
                    disabled={!canSubmit}
                    aria-label="submit tome"
                    className="inline-flex items-center gap-1.5 text-xs font-semibold py-2 px-4 rounded-md bg-purple-600 text-white hover:bg-purple-500 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                >
                    {isSubmitting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                    <span>{isSubmitting ? "Saving tome..." : "Submit"}</span>
                </button>
            </div>
        </div>
    );
};

const SaveTomeModal: React.FC<SaveTomeModalProps> = ({
    isOpen,
    onClose,
    composerCode,
    existingTomes = [],
    defaultAuthor = "operator",
    onTomeCreated,
}) => {
    const [createTome] = useMutation(CREATE_TOME_MUTATION);

    const [step, setStep] = useState<number>(0);
    const [name, setName] = useState<string>("");
    const [description, setDescription] = useState<string>("");
    const [tactic, setTactic] = useState<string>("UNSPECIFIED");
    const [author, setAuthor] = useState<string>(defaultAuthor || "operator");
    const [params, setParams] = useState<TomeParamDefInput[]>([]);
    const [eldritch, setEldritch] = useState<string>("");
    const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
    const [submitError, setSubmitError] = useState<string | null>(null);

    useEffect(() => {
        if (isOpen) {
            const draft = parseComposerScriptForTome(composerCode, existingTomes);
            setStep(0);
            setName(draft.suggestedName);
            setDescription(draft.suggestedDescription);
            setTactic(draft.suggestedTactic || "UNSPECIFIED");
            setAuthor(defaultAuthor || "operator");
            setParams(draft.detectedParams);
            setEldritch(draft.cleanedEldritch);
            setIsSubmitting(false);
            setSubmitError(null);
        }
    }, [isOpen, composerCode, existingTomes, defaultAuthor]);

    const isDuplicateName = useMemo(() => {
        const trimmed = name.trim().toLowerCase();
        if (!trimmed) return false;
        return existingTomes.some((t) => t.name.trim().toLowerCase() === trimmed);
    }, [name, existingTomes]);

    const canAdvanceFromDetails = Boolean(
        name.trim() && description.trim() && author.trim() && !isDuplicateName
    );

    const canAdvanceFromParams = useMemo(
        () => params.every((p) => p.name.trim().length > 0),
        [params]
    );

    const canSubmit = Boolean(
        canAdvanceFromDetails && canAdvanceFromParams && eldritch.trim() && !isSubmitting
    );

    const handleAddParam = () => {
        setParams((prev) => [
            ...prev,
            {
                name: "",
                label: "",
                type: "string",
                placeholder: "",
            },
        ]);
    };

    const handleUpdateParam = (
        idx: number,
        field: keyof TomeParamDefInput,
        value: string
    ) => {
        setParams((prev) =>
            prev.map((item, i) => {
                if (i !== idx) return item;
                const updated = { ...item, [field]: value };
                if (field === "name" && (!item.label || item.label === formatParamLabel(item.name))) {
                    updated.label = formatParamLabel(value);
                }
                return updated;
            })
        );
    };

    const handleRemoveParam = (idx: number) => {
        setParams((prev) => prev.filter((_, i) => i !== idx));
    };

    const handleCreateTome = async () => {
        if (!canSubmit) return;
        setIsSubmitting(true);
        setSubmitError(null);

        const paramDefsJson = serializeTomeParamDefs(params);
        const input = {
            name: name.trim(),
            description: description.trim(),
            author: author.trim(),
            tactic: tactic || "UNSPECIFIED",
            supportModel: "COMMUNITY",
            paramDefs: paramDefsJson,
            eldritch: eldritch.trim(),
        };

        try {
            let createdTome: ShellTomeItem = {
                id: `tome-custom-${Date.now()}`,
                name: input.name,
                description: input.description,
                tactic: input.tactic,
                supportModel: input.supportModel,
                paramDefs: input.paramDefs,
                eldritch: input.eldritch,
            };

            const response = await createTome({
                variables: { input },
                refetchQueries: [{ query: GET_SHELL_TOMES }],
            });
            if (response.data?.createTome) {
                createdTome = {
                    ...createdTome,
                    ...response.data.createTome,
                };
            }

            onTomeCreated?.(createdTome);
            onClose();
        } catch (err: any) {
            setSubmitError(err?.message || "Failed to save tome.");
        } finally {
            setIsSubmitting(false);
        }
    };

    if (!isOpen) {
        return null;
    }

    const renderStepBody = () => {
        if (step === 0) {
            return (
                <StepTomeDetails
                    name={name}
                    onNameChange={setName}
                    description={description}
                    onDescriptionChange={setDescription}
                    tactic={tactic}
                    onTacticChange={setTactic}
                    author={author}
                    onAuthorChange={setAuthor}
                    isDuplicateName={isDuplicateName}
                    canContinue={canAdvanceFromDetails}
                    onCancel={onClose}
                    onContinue={() => setStep(1)}
                />
            );
        }
        if (step === 1) {
            return (
                <StepTomeParameters
                    params={params}
                    onAddParam={handleAddParam}
                    onUpdateParam={handleUpdateParam}
                    onRemoveParam={handleRemoveParam}
                    canContinue={canAdvanceFromParams}
                    onBack={() => setStep(0)}
                    onContinue={() => setStep(2)}
                />
            );
        }
        return (
            <StepTomeReview
                name={name}
                description={description}
                tactic={tactic}
                author={author}
                params={params}
                eldritch={eldritch}
                onEldritchChange={setEldritch}
                submitError={submitError}
                isSubmitting={isSubmitting}
                canSubmit={canSubmit}
                onBack={() => setStep(1)}
                onSubmit={handleCreateTome}
            />
        );
    };

    return (
        <Transition show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-50" onClose={onClose}>
                <Transition.Child
                    as={Fragment}
                    enter="ease-out duration-200"
                    enterFrom="opacity-0"
                    enterTo="opacity-100"
                    leave="ease-in duration-150"
                    leaveFrom="opacity-100"
                    leaveTo="opacity-0"
                >
                    <div className="fixed inset-0 bg-black/60" aria-hidden="true" />
                </Transition.Child>

                <div className="fixed inset-0 flex items-center justify-center p-4">
                    <Transition.Child
                        as={Fragment}
                        enter="ease-out duration-200"
                        enterFrom="opacity-0 scale-95"
                        enterTo="opacity-100 scale-100"
                        leave="ease-in duration-150"
                        leaveFrom="opacity-100 scale-100"
                        leaveTo="opacity-0 scale-95"
                    >
                        <Dialog.Panel
                            data-testid="save-tome-modal"
                            className="w-full max-w-2xl rounded-lg bg-[#1c202b] border border-[#343b4d] shadow-2xl overflow-hidden"
                        >
                            {/* Modal Header */}
                            <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#2e3444]">
                                <Dialog.Title className="text-white font-semibold text-base flex items-center gap-2">
                                    <span className="relative inline-flex items-center justify-center text-purple-400">
                                        <BookOpenIcon className="w-4 h-4" />
                                        <Plus className="w-2.5 h-2.5 absolute -top-1 -right-1 bg-[#1c202b] rounded-full stroke-[2.5]" />
                                    </span>
                                    <span>Save as Tome</span>
                                </Dialog.Title>
                                <button
                                    type="button"
                                    aria-label="Close save tome modal"
                                    onClick={onClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-5 w-5" />
                                </button>
                            </div>

                            {/* Body: Left Vertical Progression + Right Step View */}
                            <form
                                id="save-tome-modal-form"
                                className="px-5 py-5"
                                onSubmit={(e) => {
                                    e.preventDefault();
                                    if (step === 2) {
                                        handleCreateTome();
                                    }
                                }}
                            >
                                <div className="flex flex-row gap-8">
                                    <div className="flex-shrink-0 mt-1 hidden sm:block w-40">
                                        <SaveTomeVerticalSteps steps={STEPS} currStep={step} />
                                    </div>
                                    <div className="flex-1 min-w-0">{renderStepBody()}</div>
                                </div>
                            </form>
                        </Dialog.Panel>
                    </Transition.Child>
                </div>
            </Dialog>
        </Transition>
    );
};

export default SaveTomeModal;
