import { Dispatch, SetStateAction, useState } from "react";
import { Heading, Switch, Textarea } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { advancedScriptFields, textareaClass } from "./constants";
import { UNIQUE_FIELD_INDEX } from "./validation";

type AdvancedStepProps = {
    scripts: string[];
    setScripts: Dispatch<SetStateAction<string[]>>;
    uniqueError: string | null;
    onBack: () => void;
    onNext: () => void;
};

export const AdvancedStep = ({ scripts, setScripts, uniqueError, onBack, onNext }: AdvancedStepProps) => {
    // Open up front when a value in this step needs fixing, so the error is visible.
    const [enabled, setEnabled] = useState(!!uniqueError);

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Advanced</h2>
                <p className="text-sm text-gray-700 italic">
                    Go-template build/setup scripts and artifact path. Defaults work for
                    standard imix builds -- only change these for custom build pipelines.
                </p>
            </div>

            <div className="flex items-center gap-2">
                <label htmlFor="enableAdvancedOptions" className="text-sm font-medium text-gray-700">
                    Enable advanced options
                </label>
                <Switch
                    id="enableAdvancedOptions"
                    colorScheme="purple"
                    isChecked={enabled}
                    onChange={() => setEnabled((value) => !value)}
                />
            </div>

            {enabled && (
                <div className="flex flex-col gap-4">
                    {advancedScriptFields.map((field, index) => (
                        <div key={field.label} className="flex flex-col gap-1">
                            <Heading size="sm">{field.label}</Heading>
                            <Textarea
                                className={textareaClass}
                                rows={field.rows}
                                value={scripts[index]}
                                onChange={(event) => setScripts((current) => current.map((value, i) => i === index ? event.target.value : value))}
                            />
                            {field.description && <p className="text-xs text-gray-500">{field.description}</p>}
                            {index === UNIQUE_FIELD_INDEX && uniqueError && <p role="alert" className="text-xs text-red-600">{uniqueError}</p>}
                        </div>
                    ))}
                </div>
            )}

            <div className="flex flex-row gap-2">
                <Button buttonVariant="ghost" onClick={onBack} aria-label="back button">Back</Button>
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    disabled={!!uniqueError}
                    aria-label="continue advanced step"
                    onClick={onNext}
                >
                    Continue
                </Button>
            </div>
        </div>
    );
};
