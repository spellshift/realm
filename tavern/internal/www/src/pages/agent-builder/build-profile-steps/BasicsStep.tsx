import { ReactNode } from "react";
import { Input, Textarea } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";

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

const Field = ({ htmlFor, label, children }: { htmlFor: string; label: string; children: ReactNode }) => (
    <div className="flex flex-col gap-1">
        <label htmlFor={htmlFor} className="font-medium text-gray-700">{label}</label>
        {children}
    </div>
);

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
            <Field htmlFor="profileName" label="Name">
                <Input
                    id="profileName"
                    name="profileName"
                    size="sm"
                    colorScheme="purple"
                    placeholder="e.g. Stealth DNS"
                    value={name}
                    onChange={(e) => onNameChange(e.target.value)}
                />
            </Field>
            <Field htmlFor="profileDescription" label="Description">
                <Textarea
                    id="profileDescription"
                    size="sm"
                    colorScheme="purple"
                    rows={2}
                    placeholder="What is this profile for?"
                    value={description}
                    onChange={(e) => onDescriptionChange(e.target.value)}
                />
            </Field>
            <Field htmlFor="buildImage" label="Build image">
                <Input
                    id="buildImage"
                    name="buildImage"
                    size="sm"
                    colorScheme="purple"
                    placeholder="spellshift/devcontainer:main"
                    value={buildImage}
                    onChange={(e) => onBuildImageChange(e.target.value)}
                />
            </Field>
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
