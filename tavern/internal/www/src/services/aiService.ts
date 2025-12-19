
export const checkAI = async (): Promise<boolean> => {
    if (!window.ai) return false;
    try {
        const capabilities = await window.ai.languageModel.capabilities();
        return capabilities.available !== "no";
    } catch (e) {
        console.error("Error checking AI capabilities:", e);
        return false;
    }
};

export interface GeneratedTomeData {
    name: string;
    description: string;
    author: string;
    tactic: string;
    paramDefs: any[]; // strict typing would be better but any is fine for JSON parsing
    eldritch: string;
}

export const generateTome = async (prompt: string): Promise<GeneratedTomeData> => {
    if (!window.ai) throw new Error("AI not available");

    const systemPrompt = `You are an expert in creating "Realm Tomes" for the Tavern security platform.
A Tome consists of metadata and an execution script in the "Eldritch" language (a Starlark/Python dialect).

You must generate a valid JSON object with the following structure:
{
  "name": "Tome Name",
  "description": "Description of what the tome does",
  "author": "AI Assistant",
  "tactic": "RECON",
  "paramDefs": [
    { "name": "param_name", "label": "Parameter Label", "type": "string", "placeholder": "example value" }
  ],
  "eldritch": "print('hello world')"
}

Valid tactics are: RECON, RESOURCE_DEVELOPMENT, INITIAL_ACCESS, EXECUTION, PERSISTENCE, PRIVILEGE_ESCALATION, DEFENSE_EVASION, CREDENTIAL_ACCESS, DISCOVERY, LATERAL_MOVEMENT, COLLECTION, COMMAND_AND_CONTROL, EXFILTRATION, IMPACT.

The "eldritch" script supports Python-like syntax. Use 'print()' for output. Input parameters are available in the 'input_params' dictionary, e.g., input_params['param_name'].

Example script that lists files:
usernfo = sys.get_user()
def list_files(path):
    res = file.list(path)
    for f in res:
        print(f['absolute_path'])
list_files(input_params['path'])

Generate only the JSON object. Do not include markdown formatting.`;

    try {
        const session = await window.ai.languageModel.create({ systemPrompt });
        const result = await session.prompt(prompt);
        session.destroy();

        // Attempt to parse JSON. If it fails, try to strip markdown blocks.
        let cleaned = result.trim();
        if (cleaned.startsWith("```json")) {
            cleaned = cleaned.replace(/^```json/, "").replace(/```$/, "");
        } else if (cleaned.startsWith("```")) {
             cleaned = cleaned.replace(/^```/, "").replace(/```$/, "");
        }

        return JSON.parse(cleaned);
    } catch (e) {
        console.error("AI generation failed:", e);
        throw e;
    }
};
