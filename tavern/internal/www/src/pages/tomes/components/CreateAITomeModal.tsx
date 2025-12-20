import { FC, useState } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useCreateTome } from "../hooks/useCreateTome";
import { useMutation } from "@apollo/client";
import { GENERATE_TOME_AI } from "../../../utils/queries";

type Props = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
};

const CreateAITomeModal: FC<Props> = ({ isOpen, setOpen }) => {
    const [prompt, setPrompt] = useState("");
    const [generatedData, setGeneratedData] = useState<any>(null);
    const [generationError, setGenerationError] = useState("");
    const { createTome, loading: isSaving, error: saveError } = useCreateTome();
    const [generateTomeAI, { loading: isGenerating }] = useMutation(GENERATE_TOME_AI);

    const handleGenerate = async () => {
        setGenerationError("");
        try {
            const { data } = await generateTomeAI({ variables: { prompt } });
            setGeneratedData(data.generateTomeAI);
        } catch (e: any) {
            setGenerationError(e.message || "Failed to generate tome");
        }
    };

    const handleSave = async () => {
        if (!generatedData) return;
        try {
            // Prepare input for mutation
            const input = {
                name: generatedData.name,
                description: generatedData.description || "",
                author: generatedData.author || "AI",
                tactic: generatedData.tactic || "UNSPECIFIED",
                supportModel: "FIRST_PARTY",
                paramDefs: generatedData.paramDefs || "[]",
                eldritch: generatedData.eldritch,
            };
            await createTome(input);
            setOpen(false);
        } catch (e) {
            console.error("Failed to save tome", e);
        }
    };

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="lg">
            <div className="flex flex-col gap-6 p-4">
                <h3 className="text-xl font-semibold text-gray-900">Create Tome with AI</h3>

                {!generatedData ? (
                    <div className="flex flex-col gap-4">
                        <textarea
                            className="w-full h-32 p-2 border rounded-md"
                            placeholder="Describe the tome you want to create (e.g. 'A tome that lists all files in /tmp')"
                            value={prompt}
                            onChange={(e) => setPrompt(e.target.value)}
                        />
                        <div className="flex justify-end gap-2">
                             <Button onClick={() => setOpen(false)} buttonStyle={{ color: "gray" }} buttonVariant="outline">Cancel</Button>
                             <Button onClick={handleGenerate} disabled={!prompt || isGenerating} buttonStyle={{ color: "purple" }}>
                                {isGenerating ? "Generating..." : "Generate"}
                             </Button>
                        </div>
                        {generationError && <div className="text-red-500">{generationError}</div>}
                    </div>
                ) : (
                    <div className="flex flex-col gap-4">
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700">Name</label>
                                <input type="text" className="mt-1 block w-full border rounded-md p-1" value={generatedData.name} onChange={(e) => setGeneratedData({...generatedData, name: e.target.value})} />
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-700">Tactic</label>
                                <input type="text" className="mt-1 block w-full border rounded-md p-1" value={generatedData.tactic} onChange={(e) => setGeneratedData({...generatedData, tactic: e.target.value})} />
                            </div>
                        </div>
                        <div>
                             <label className="block text-sm font-medium text-gray-700">Description</label>
                             <textarea className="mt-1 block w-full border rounded-md p-1" value={generatedData.description} onChange={(e) => setGeneratedData({...generatedData, description: e.target.value})} />
                        </div>
                        <div>
                             <label className="block text-sm font-medium text-gray-700">Script (Eldritch)</label>
                             <textarea className="mt-1 block w-full border rounded-md p-1 font-mono h-48" value={generatedData.eldritch} onChange={(e) => setGeneratedData({...generatedData, eldritch: e.target.value})} />
                        </div>

                        <div className="flex justify-end gap-2">
                            <Button onClick={() => setGeneratedData(null)} buttonStyle={{ color: "gray" }} buttonVariant="outline">Back</Button>
                            <Button onClick={handleSave} disabled={isSaving} buttonStyle={{ color: "purple" }}>
                                {isSaving ? "Saving..." : "Save Tome"}
                            </Button>
                        </div>
                         {saveError && <div className="text-red-500">{saveError.message}</div>}
                    </div>
                )}
            </div>
        </Modal>
    );
};

export default CreateAITomeModal;
