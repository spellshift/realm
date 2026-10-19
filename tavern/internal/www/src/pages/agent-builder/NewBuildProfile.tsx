import { useState } from "react";
import { useMutation } from "@apollo/client";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { CREATE_BUILD_PROFILE, GET_BUILD_PROFILES, mapProfile, profileInput, ProfileRecord } from "./queries";
import { safelyJsonParse } from "../../utils/utils";
import { Heading, Textarea } from "@chakra-ui/react";
import { TrashIcon, PlusIcon, ChevronDownIcon, ChevronUpIcon, ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import Button from "../../components/tavern-base-ui/button/Button";
import FormTextField from "../../components/tavern-base-ui/FormTextField";
import SingleDropdownSelector from "../../components/tavern-base-ui/SingleDropdownSelector";
import { FormRadioGroup } from "../../components/tavern-base-ui/FormRadioGroup";
import Modal from "../../components/tavern-base-ui/Modal";
import { SupportedTransports } from "../../utils/enums";
import {
    TRANSPORT_TYPES,
    DEFAULT_TRANSPORT,
    Tome,
    DEFAULT_SETUP_SCRIPT,
    TransportConfig,
    TomeRef,
    BuildProfile,
    NO_INSTALL_WARNING,
} from "./types";

const textareaClass =
    "block w-full p-2 font-mono text-xs placeholder-gray-500 rounded-md border-0 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600";

const advancedScriptFields = [
    { label: "Build script", defaultValue: "{{.BuildCommand}}", rows: 2 },
    { label: "Artifact path", defaultValue: "{{.ArtifactPath}}", rows: 2 },
    { label: "Prebuild script", defaultValue: "echo 'no prebuild set'", rows: 2 },
    { label: "Setup script", defaultValue: DEFAULT_SETUP_SCRIPT, rows: 3 },
    { label: "Postbuild script", defaultValue: "echo 'no postbuild set'", rows: 2 },
    {
        label: "Unique (JSON, optional)",
        defaultValue: "",
        rows: 2,
        description: 'Controls how the agent identifies its host. Enter an ordered JSON array of selectors, such as [{"type":"env"},{"type":"file"}]. Leave blank to use the default selectors.',
    },
];

/**
 * New build profile authoring screen, reached via "+ New profile" from the
 * profile-picker step of the wizard. Fields mirror the
 * BuildProfile ent schema (tavern/internal/ent/schema/buildprofile.go):
 * name, description, transports (ordered list of {type, uri, interval}),
 * build_image, tomes (ordered list of {tome, params}), and the Go-template
 * script fields, which are collapsed under "Advanced" per the earlier
 * design decision -- most operators will never touch them.
 */
export const NewBuildProfile = ({
    catalog,
    onCancel,
    onSave,
}: {
    catalog: Tome[];
    onCancel: () => void;
    onSave: (profile: BuildProfile) => Promise<void> | void;
}) => {
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [buildImage, setBuildImage] = useState("spellshift/devcontainer:main");
    const [transports, setTransports] = useState<TransportConfig[]>([DEFAULT_TRANSPORT]);
    const [tomes, setTomes] = useState<TomeRef[]>([]);
    const [addingTome, setAddingTome] = useState(false);
    const [tomeToAdd, setTomeToAdd] = useState<Tome | null>(null);
    const [tomeParams, setTomeParams] = useState("{}");
    const [scripts, setScripts] = useState(advancedScriptFields.map((field, index) => field.defaultValue));
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);
    const [savedProfileRecord, setSavedProfileRecord] = useState<ProfileRecord | null>(null);
    const [createProfile, { loading: saving }] = useMutation<{ createBuildProfile: ProfileRecord }>(CREATE_BUILD_PROFILE, {
        refetchQueries: [{ query: GET_BUILD_PROFILES }],
    });
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [transportModalOpen, setTransportModalOpen] = useState(false);
    const [transportDraft, setTransportDraft] = useState<TransportConfig | null>(null);

    const openAddTransport = () => {
        setTransportDraft({ id: `t${Date.now()}`, type: SupportedTransports.HTTP1, uri: "", interval: 30 });
        setTransportModalOpen(true);
    };
    const openEditTransport = (transport: TransportConfig) => {
        setTransportDraft({ ...transport });
        setTransportModalOpen(true);
    };
    const saveTransport = () => {
        if (!transportDraft?.uri.trim() || !Number.isInteger(transportDraft.interval) || transportDraft.interval <= 0) return;
        const next = { ...transportDraft, uri: transportDraft.uri.trim() };
        setTransports((list) => list.some((transport) => transport.id === next.id)
            ? list.map((transport) => transport.id === next.id ? next : transport)
            : [...list, next]);
        setTransportModalOpen(false);
    };
    const removeTransport = (id: string) => setTransports((t) => t.filter((x) => x.id !== id));

    const confirmAddTome = () => {
        if (!tomeToAdd) return;
        try {
            const params = JSON.parse(tomeParams);
            if (!params || Array.isArray(params) || typeof params !== "object") throw new Error("Tome parameters must be a JSON object.");
        } catch (error) {
            setError(error instanceof Error ? error.message : String(error));
            return;
        }
        setError(null);
        setTomes((list) => [
            ...list,
            { id: `tr${list.length + 1}-${Date.now()}`, tomeId: tomeToAdd.id, tomeName: tomeToAdd.name, paramSummary: tomeParams === "{}" ? "No params" : tomeParams, params: tomeParams, tactic: tomeToAdd.tactic },
        ]);
        setAddingTome(false);
    };

    const hasPersistenceTome = tomes.some((t) => t.tactic === "PERSISTENCE");

    const handleSave = async () => {
        if (!name.trim() || saving || submitting) return;
        setError(null);
        setSubmitting(true);
        try {
            if (scripts[5].trim()) JSON.parse(scripts[5]);
            let profileRecord = savedProfileRecord;
            if (!profileRecord) {
                const profile: BuildProfile = {
                    id: "", name, description, transports, buildImage,
                    tomes, tomeCount: tomes.length, hasPersistenceTome,
                    buildScript: scripts[0], artifactPath: scripts[1], prebuildscript: scripts[2],
                    setupscript: scripts[3], postbuildscript: scripts[4], unique: scripts[5],
                };
                const { data } = await createProfile({ variables: { input: profileInput(profile) } });
                if (!data?.createBuildProfile) throw new Error("The server did not return a build profile.");
                profileRecord = data.createBuildProfile;
                setSavedProfileRecord(profileRecord);
            }
            await onSave(mapProfile(profileRecord, catalog));
        } catch (error) {
            setError(error instanceof Error ? error.message : String(error));
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
        <div className="flex flex-col gap-4 w-full min-w-0 max-w-xl">
            {error && <AlertError label="Unable to save profile" details={error} />}
            <div className="flex flex-col gap-1">
                <Heading size="sm">New build profile</Heading>
                <p className="text-sm text-gray-500">
                    Set the defaults operators will use when creating a build.
                </p>
            </div>

            <div className="flex flex-col gap-4 rounded-md border border-gray-200 p-4">
                <FormTextField
                    htmlFor="profileName"
                    label="Name"
                    placeholder="e.g. Stealth DNS"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                />
                <div>
                    <label className="block text-sm font-medium text-gray-900 mb-1">Description</label>
                    <Textarea
                        className={textareaClass + " font-sans"}
                        rows={2}
                        placeholder="What is this profile for?"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                    />
                </div>
                <FormTextField
                    htmlFor="buildImage"
                    label="Build image"
                    placeholder="spellshift/devcontainer:main"
                    value={buildImage}
                    onChange={(e) => setBuildImage(e.target.value)}
                />
            </div>

            <div className="flex flex-col gap-3 rounded-md border border-gray-200 p-4">
                <Heading size="sm">Transports ({transports.length})</Heading>
                <div className="flex flex-col gap-2">
                    {transports.map((t) => (
                        <div key={t.id} className="flex items-center gap-3 rounded-md border border-gray-200 p-3 min-w-0">
                            <div className="flex flex-col min-w-0 flex-1">
                                <span className="text-sm font-medium text-gray-900">
                                    {TRANSPORT_TYPES.find((type) => type.value === t.type)?.label} · Every {t.interval}s
                                </span>
                                <span className="text-xs text-gray-500 break-all">{t.uri}</span>
                            </div>
                            <Button buttonVariant="ghost" buttonStyle={{ color: "purple", size: "sm" }} onClick={() => openEditTransport(t)}>
                                Edit
                            </Button>
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "square_sm" }}
                                onClick={() => removeTransport(t.id)}
                                disabled={transports.length === 1}
                                aria-label="remove transport"
                            >
                                <TrashIcon className="w-4 h-4" />
                            </Button>
                        </div>
                    ))}
                </div>
                <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "purple", size: "sm" }}
                    leftIcon={<PlusIcon className="w-4 h-4" />}
                    onClick={openAddTransport}
                    className="w-fit"
                >
                    Add transport
                </Button>
            </div>

            <div className="flex flex-col gap-3 rounded-md border border-gray-200 p-4">
                <Heading size="sm">Bundled tomes ({tomes.length})</Heading>
                {!hasPersistenceTome && (
                    <div className="rounded-md bg-amber-50 py-2 px-3">
                        <div className="flex flex-row gap-2">
                            <ExclamationTriangleIcon className="h-5 w-5 text-amber-400 flex-shrink-0" aria-hidden="true" />
                            <p className="text-sm text-amber-700">
                                {NO_INSTALL_WARNING}
                            </p>
                        </div>
                    </div>
                )}
                {tomes.length === 0 && !addingTome && (
                    <p className="text-sm text-gray-500">No tomes bundled yet.</p>
                )}
                <div className="flex flex-col gap-2">
                    {tomes.map((t) => (
                        <div key={t.id} className="flex flex-row items-center justify-between rounded-md border border-gray-200 p-3">
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
                                <TrashIcon className="w-4 h-4" />
                            </Button>
                        </div>
                    ))}
                </div>

                {addingTome ? (
                    <div className="flex flex-col gap-3 rounded-md border border-gray-200 p-3">
                        <FormRadioGroup
                            label="Choose a tome"
                            data={catalog}
                            selected={tomeToAdd}
                            setSelected={(tome: Tome) => {
                                setTomeToAdd(tome);
                                const { params: definitions } = safelyJsonParse(tome.paramDefs || "[]");
                                setTomeParams(JSON.stringify(Object.fromEntries(definitions.map((param: { name: string; value?: string }) => [param.name, param.value || ""])), null, 2));
                            }}
                        />
                        {catalog.length === 0 && <p className="text-sm text-gray-500">No tomes available. Import a tome before adding it to this profile.</p>}
                        {tomeToAdd && <FormTextField htmlFor="tomeParams" label="Parameters (JSON object)" value={tomeParams} onChange={(event) => setTomeParams(event.target.value)} />}
                        <div className="flex flex-row gap-2">
                            <Button buttonVariant="ghost" buttonStyle={{ size: "sm" }} onClick={() => setAddingTome(false)}>
                                Cancel
                            </Button>
                            <Button buttonStyle={{ size: "sm" }} onClick={confirmAddTome} disabled={!tomeToAdd}>
                                Add to profile
                            </Button>
                        </div>
                    </div>
                ) : (
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "purple", size: "sm" }}
                        leftIcon={<PlusIcon className="w-4 h-4" />}
                        onClick={() => setAddingTome(true)}
                        className="w-fit"
                    >
                        Add tome
                    </Button>
                )}
            </div>

            <div className="flex flex-col gap-2 border-t border-gray-200 pt-4">
                <button
                    type="button"
                    className="flex flex-row items-center gap-1 text-sm font-medium text-purple-700 text-left w-fit"
                    onClick={() => setShowAdvanced((v) => !v)}
                >
                    Advanced
                    {showAdvanced ? <ChevronUpIcon className="w-4 h-4" /> : <ChevronDownIcon className="w-4 h-4" />}
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
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onCancel}>Cancel</Button>
                <Button onClick={handleSave} isLoading={saving || submitting} disabled={!name.trim() || !buildImage.trim() || !scripts[0].trim() || !scripts[1].trim()}>Save profile</Button>
            </div>
        </div>
        <Modal isOpen={transportModalOpen} setOpen={setTransportModalOpen} size="sm">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        {transports.some((transport) => transport.id === transportDraft?.id) ? "Edit transport" : "Add transport"}
                    </h3>
                    <p className="mt-1 text-sm text-gray-500">Configure the transport used by this build profile.</p>
                </div>
                {transportDraft && (
                    <div className="flex flex-col gap-4">
                        <div>
                            <Heading size="sm" className="mb-1">Type</Heading>
                            <SingleDropdownSelector
                                key={`${transportDraft.id}-${transportDraft.type}`}
                                label="Type"
                                options={TRANSPORT_TYPES.map((type) => ({ label: type.label, value: type.value }))}
                                defaultValue={TRANSPORT_TYPES.find((type) => type.value === transportDraft.type)}
                                setSelectedOption={(option: { value: TransportConfig["type"] }) =>
                                    setTransportDraft((draft) => draft && { ...draft, type: option.value })}
                            />
                        </div>
                        <FormTextField
                            htmlFor="transportUri"
                            label="URI"
                            placeholder="http://127.0.0.1:8000"
                            value={transportDraft.uri}
                            onChange={(event) => setTransportDraft((draft) => draft && { ...draft, uri: event.target.value })}
                        />
                        <FormTextField
                            htmlFor="transportInterval"
                            label="Interval (s)"
                            type="number"
                            min={1}
                            value={transportDraft.interval}
                            onChange={(event) => setTransportDraft((draft) => draft && { ...draft, interval: Number(event.target.value) })}
                        />
                    </div>
                )}
                <div className="flex flex-row gap-2">
                    <Button buttonVariant="ghost" onClick={() => setTransportModalOpen(false)}>Cancel</Button>
                    <Button
                        onClick={saveTransport}
                        disabled={!transportDraft || !transportDraft.uri.trim() || !Number.isInteger(transportDraft.interval) || transportDraft.interval <= 0}
                    >
                        Save transport
                    </Button>
                </div>
            </div>
        </Modal>
        </>
    );
};

export default NewBuildProfile;
