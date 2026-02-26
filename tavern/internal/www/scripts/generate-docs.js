const fs = require('fs');
const path = require('path');

const inputPath = path.resolve(__dirname, '../../../../docs/_docs/user-guide/eldritch.md');
const outputPath = path.resolve(__dirname, '../src/assets/eldritch-docs.json');

// Ensure output directory exists
const outputDir = path.dirname(outputPath);
if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

console.log(`Reading from: ${inputPath}`);
console.log(`Writing to: ${outputPath}`);

try {
    const content = fs.readFileSync(inputPath, 'utf-8');
    const docs = {};

    // Simple parsing logic
    // We look for "### function_name"
    // Then capture the next code block as signature
    // Then capture the text until the next header or end of file as description

    const lines = content.split('\n');
    let currentFunction = null;
    let currentSignature = '';
    let currentDescription = [];
    let capturingDescription = false;

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();

        if (line.startsWith('### ')) {
            // New function found, save previous one if exists
            if (currentFunction) {
                docs[currentFunction] = {
                    signature: currentSignature,
                    description: currentDescription.join(' ').trim()
                };
            }

            // Start new function
            // format: ### agent.get_config
            const match = line.match(/^###\s+([\w\.]+)/);
            if (match) {
                currentFunction = match[1];
                currentSignature = '';
                currentDescription = [];
                capturingDescription = false;
            } else {
                currentFunction = null; // invalid header format or not a function header we care about
            }
            continue;
        }

        if (currentFunction) {
            // Try to capture signature
            // It might be in inline code `...` or block ```python ... ```
            // The markdown seems to use inline code `function() -> type` right after header mostly

            if (!currentSignature && line.startsWith('`') && line.endsWith('`')) {
                currentSignature = line.slice(1, -1);
                capturingDescription = true;
                continue;
            }

            // Handle multiline code block for signature if present (rare in the snippet I saw but possible)
            // The snippet shows `function` so I will stick to that first.
            // If the signature is missing, maybe it is in the description?

            // If we have signature, subsequent text is description
            if (capturingDescription) {
                 // Stop capturing if we hit another header (handled by the loop start)
                 // Just append lines
                 if (line.length > 0) {
                     currentDescription.push(line);
                 }
            } else if (!currentSignature && line.length > 0) {
                 // If we haven't found a signature yet but found text, assume signature is missing or in a different format
                 // For now, let's just treat it as description start if it doesn't look like code
                 // But looking at the file, signature is almost always immediately after header in backticks
            }
        }
    }

    // Add the last function
    if (currentFunction) {
        docs[currentFunction] = {
            signature: currentSignature,
            description: currentDescription.join(' ').trim()
        };
    }

    fs.writeFileSync(outputPath, JSON.stringify(docs, null, 2));
    console.log(`Successfully generated docs for ${Object.keys(docs).length} functions.`);

} catch (error) {
    console.error('Error generating docs:', error);
    process.exit(1);
}
