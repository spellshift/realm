import { DEFAULT_SETUP_SCRIPT } from "../types";

export const textareaClass =
    "block w-full p-2 font-mono text-xs placeholder-gray-500 rounded-md border-0 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-purple-600";

export const advancedScriptFields = [
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
