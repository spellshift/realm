import { useState } from "react";
import { useMutation } from "@apollo/client";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { FormSteps } from "../../components/form-steps";
import { CREATE_BUILD_PROFILE, UPDATE_BUILD_PROFILE, GET_BUILD_PROFILES, mapProfile, profileInput, ProfileRecord } from "./queries";
import { BasicsStep, TransportsStep, TomesStep, AdvancedStep, ReviewStep, advancedScriptFields, UNIQUE_FIELD_INDEX, uniqueJsonError } from "./build-profile-steps";
import { DEFAULT_TRANSPORT, Tome, TransportConfig, TomeRef, BuildProfile } from "./types";

const STEPS = [
    { name: "Basics", description: "Step 1", href: "#", step: 0 },
    { name: "Transports", description: "Step 2", href: "#", step: 1 },
    { name: "Tomes", description: "Step 3", href: "#", step: 2 },
    { name: "Advanced", description: "Step 4", href: "#", step: 3 },
    { name: "Review", description: "Step 5", href: "#", step: 4 },
];

/**
 * Build profile authoring screen -- used both to create a new profile
 * (reached via "+ New profile" from the profile-picker step of the wizard)
 * and, when `existingProfile` is passed, to edit one in place from the
 * Profiles tab. Fields mirror the BuildProfile ent schema
 * (tavern/internal/ent/schema/buildprofile.go): name, description,
 * transports (ordered list of {type, uri, interval}), build_image, tomes
 * (ordered list of {tome, params}), and the Go-template script fields,
 * which are collapsed under "Advanced" per the earlier design decision --
 * most operators will never touch them.
 *
 * Presented as a phased stepper (Basics -> Transports -> Tomes -> Review),
 * matching the pattern used elsewhere in Tavern (e.g. the agent-build
 * wizard and create-quest modal). This component owns the form state and
 * the save mutation; each step lives in `build-profile-steps/`.
 */
export const NewBuildProfile = ({
    catalog,
    existingProfile,
    onCancel,
    onSave,
}: {
    catalog: Tome[];
    existingProfile?: BuildProfile;
    onCancel: () => void;
    onSave: (profile: BuildProfile) => Promise<void> | void;
}) => {
    const isEditing = !!existingProfile;
    const [currStep, setCurrStep] = useState(0);
    const [name, setName] = useState(existingProfile?.name ?? "");
    const [description, setDescription] = useState(existingProfile?.description ?? "");
    const [buildImage, setBuildImage] = useState(existingProfile?.buildImage ?? "spellshift/devcontainer:main");
    const [transports, setTransports] = useState<TransportConfig[]>(existingProfile?.transports ?? [DEFAULT_TRANSPORT]);
    const [tomes, setTomes] = useState<TomeRef[]>(existingProfile?.tomes ?? []);
    const [scripts, setScripts] = useState(
        existingProfile
            ? [existingProfile.buildScript, existingProfile.artifactPath, existingProfile.prebuildscript, existingProfile.setupscript, existingProfile.postbuildscript, existingProfile.unique ?? ""]
            : advancedScriptFields.map((field) => field.defaultValue)
    );
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);
    const [savedProfileRecord, setSavedProfileRecord] = useState<ProfileRecord | null>(null);
    const [createProfile, { loading: creating }] = useMutation<{ createBuildProfile: ProfileRecord }>(CREATE_BUILD_PROFILE, {
        refetchQueries: [{ query: GET_BUILD_PROFILES }],
    });
    const [updateProfile, { loading: updating }] = useMutation<{ updateBuildProfile: ProfileRecord }>(UPDATE_BUILD_PROFILE, {
        refetchQueries: [{ query: GET_BUILD_PROFILES }],
    });
    const saving = creating || updating || submitting;

    const hasPersistenceTome = tomes.some((t) => t.tactic === "PERSISTENCE");
    const basicsValid = !!name.trim() && !!buildImage.trim();
    const uniqueError = uniqueJsonError(scripts[UNIQUE_FIELD_INDEX]);

    const handleSave = async () => {
        if (!name.trim() || saving || uniqueError) return;
        setError(null);
        setSubmitting(true);
        try {
            let profileRecord = savedProfileRecord;
            if (!profileRecord) {
                const profile: BuildProfile = {
                    id: existingProfile?.id ?? "", name, description, transports, buildImage,
                    tomes, tomeCount: tomes.length, hasPersistenceTome,
                    buildScript: scripts[0], artifactPath: scripts[1], prebuildscript: scripts[2],
                    setupscript: scripts[3], postbuildscript: scripts[4], unique: scripts[5],
                };
                if (existingProfile) {
                    const { data } = await updateProfile({ variables: { profileID: existingProfile.id, input: profileInput(profile) } });
                    if (!data?.updateBuildProfile) throw new Error("The server did not return a build profile.");
                    profileRecord = data.updateBuildProfile;
                } else {
                    const { data } = await createProfile({ variables: { input: profileInput(profile) } });
                    if (!data?.createBuildProfile) throw new Error("The server did not return a build profile.");
                    profileRecord = data.createBuildProfile;
                }
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
        <div className="flex flex-col gap-4 w-full min-w-0">
            {error && <AlertError label="Unable to save profile" details={error} />}

            <div className="flex flex-row gap-6">
                <div className="flex-shrink-0 mt-2 lg:block hidden">
                    <FormSteps steps={STEPS} currStep={currStep} />
                </div>
                <div className="flex-1 min-w-0">
                    {currStep === 0 && (
                        <BasicsStep
                            name={name}
                            description={description}
                            buildImage={buildImage}
                            onNameChange={setName}
                            onDescriptionChange={setDescription}
                            onBuildImageChange={setBuildImage}
                            canContinue={basicsValid}
                            onCancel={onCancel}
                            onNext={() => setCurrStep(1)}
                        />
                    )}
                    {currStep === 1 && (
                        <TransportsStep
                            transports={transports}
                            setTransports={setTransports}
                            onBack={() => setCurrStep(0)}
                            onNext={() => setCurrStep(2)}
                        />
                    )}
                    {currStep === 2 && (
                        <TomesStep
                            catalog={catalog}
                            tomes={tomes}
                            setTomes={setTomes}
                            hasPersistenceTome={hasPersistenceTome}
                            onError={setError}
                            onBack={() => setCurrStep(1)}
                            onNext={() => setCurrStep(3)}
                        />
                    )}
                    {currStep === 3 && (
                        <AdvancedStep
                            scripts={scripts}
                            setScripts={setScripts}
                            uniqueError={uniqueError}
                            onBack={() => setCurrStep(2)}
                            onNext={() => setCurrStep(4)}
                        />
                    )}
                    {currStep === 4 && (
                        <ReviewStep
                            name={name}
                            description={description}
                            buildImage={buildImage}
                            transports={transports}
                            tomes={tomes}
                            hasPersistenceTome={hasPersistenceTome}
                            isEditing={isEditing}
                            saving={saving}
                            canSave={basicsValid && !!scripts[0].trim() && !!scripts[1].trim() && !uniqueError}
                            onBack={() => setCurrStep(3)}
                            onSave={handleSave}
                        />
                    )}
                </div>
            </div>
        </div>
    );
};

export default NewBuildProfile;
