import * as vscode from 'vscode';
import * as https from 'https';

export class GeminiClient {

    public async generateTome(userPrompt: string, context: string): Promise<string> {
        const config = vscode.workspace.getConfiguration('eldritch');
        const apiKey = config.get<string>('geminiApiKey', '');
        const modelName = config.get<string>('modelName', 'gemini-2.0-flash-exp');

        if (!apiKey) {
            throw new Error("Gemini API Key is not configured. Please set 'eldritch.geminiApiKey' in settings.");
        }

        const systemPrompt = `
${context}

You are an Eldritch Tome Generator. Your task is to generate a valid Eldritch Tome based on the user's request.
A Tome consists of two files: 'metadata.yml' and 'main.eldritch'.

Output the response in the following format exactly:

---BEGIN METADATA---
<metadata.yml content>
---END METADATA---

---BEGIN CODE---
<main.eldritch content>
---END CODE---

Ensure the code is valid Eldritch (Pythonic Starlark).
Do not include any other conversational text. Just the blocks.
`;

        const payload = {
            contents: [{
                parts: [{ text: userPrompt }]
            }],
            systemInstruction: {
                parts: [{ text: systemPrompt }]
            }
        };

        const url = `https://generativelanguage.googleapis.com/v1beta/models/${modelName}:generateContent?key=${apiKey}`;

        return this.postRequest(url, payload);
    }

    private postRequest(url: string, payload: any): Promise<string> {
        return new Promise((resolve, reject) => {
            const data = JSON.stringify(payload);
            const urlObj = new URL(url);
            const options = {
                hostname: urlObj.hostname,
                path: urlObj.pathname + urlObj.search,
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(data)
                }
            };

            const req = https.request(options, (res) => {
                let body = '';
                res.on('data', (chunk) => body += chunk);
                res.on('end', () => {
                    if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
                        try {
                            const responseJson = JSON.parse(body);
                            // Navigate to candidates[0].content.parts[0].text
                            const text = responseJson.candidates?.[0]?.content?.parts?.[0]?.text;
                            if (text) {
                                resolve(text);
                            } else {
                                reject(new Error("Invalid response format from Gemini: " + body));
                            }
                        } catch (e) {
                            reject(new Error("Failed to parse Gemini response: " + body));
                        }
                    } else {
                        reject(new Error(`Gemini API Error (${res.statusCode}): ${body}`));
                    }
                });
            });

            req.on('error', (e) => reject(e));
            req.write(data);
            req.end();
        });
    }
}
