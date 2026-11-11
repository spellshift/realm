import { Textarea } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import { textareaClass } from "./constants";

type BasicsStepProps = {
    name: string;
    description: string;
    buildImage: string;
    onNameChange: (value: string) => void;
    onDescriptionChange: (value: string) => void;
    onBuildImageChange: (value: string) => void;
    canContinue: boolean;
    onCancel: () => void;
    onNext: () => void;
};

export const BasicsStep = ({
    name,
    description,
    buildImage,
    onNameChange,
    onDescriptionChange,
    onBuildImageChange,
    canContinue,
    onCancel,
    onNext,
}: BasicsStepProps) => (
    <div className="flex flex-col gap-6">
        <div className="flex flex-col gap-1">
            <h2 className="text-xl font-semibold text-gray-900">Basics</h2>
            <p className="text-sm text-gray-700 italic">Name the profile and set its build image.</p>
        </div>
        <div className="flex flex-col gap-4">
            <FormTextField
                htmlFor="profileName"
                label="Name"
                placeholder="e.g. Stealth DNS"
                value={name}
                onChange={(e) => onNameChange(e.target.value)}
            />
            <div>
                <label className="block text-sm font-medium text-gray-900 mb-1">Description</label>
                <Textarea
                    className={textareaClass + " font-sans"}
                    rows={2}
                    placeholder="What is this profile for?"
                    value={description}
                    onChange={(e) => onDescriptionChange(e.target.value)}
                />
            </div>
            <FormTextField
                htmlFor="buildImage"
                label="Build image"
                placeholder="spellshift/devcontainer:main"
                value={buildImage}
                onChange={(e) => onBuildImageChange(e.target.value)}
            />
        </div>
        <div className="flex flex-row gap-2">
            <Button buttonVariant="ghost" onClick={onCancel}>Cancel</Button>
            <Button
                buttonStyle={{ color: "purple", size: "md" }}
                disabled={!canContinue}
                aria-label="continue basics step"
                onClick={onNext}
            >
                Continue
            </Button>
        </div>
    </div>
);
