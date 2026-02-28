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

    const lines = content.split('\n');
    let currentFunction = null;
    let currentLibrary = null;
    let currentSignature = '';
    let currentDescription = [];
    let capturingDescription = false;
    let inLibrarySection = false;

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();

        if (line.startsWith('## ')) {
            // New library section
            const match = line.match(/^##\s+(\w+)/);
            if (match) {
                const libName = match[1].toLowerCase();

                const ignoreHeaders = ['examples', 'data', 'error', 'built-ins', 'standard'];
                if (!ignoreHeaders.includes(libName)) {
                    // Save previous function if any
                    if (currentFunction) {
                        docs[currentFunction] = {
                            signature: currentSignature,
                            description: currentDescription.join('\n').trim()
                        };
                        currentFunction = null;
                    }

                    currentLibrary = libName;
                    inLibrarySection = true;
                    currentDescription = [];
                    capturingDescription = true;
                } else {
                    inLibrarySection = false;
                    currentLibrary = null;
                }
            } else {
                inLibrarySection = false;
                currentLibrary = null;
            }
            continue;
        }

        if (inLibrarySection && !line.startsWith('### ')) {
            if (line.length > 0) {
                currentDescription.push(line);
            }
            continue;
        }

        if (line.startsWith('### ')) {
            // If we were capturing a library description, save it
            if (inLibrarySection && currentLibrary) {
                docs[currentLibrary] = {
                    signature: currentLibrary, // Use a dummy signature or just name
                    description: currentDescription.join('\n').trim()
                };
                inLibrarySection = false;
                currentLibrary = null;
            }

            // New function found, save previous one if exists
            if (currentFunction) {
                docs[currentFunction] = {
                    signature: currentSignature,
                    description: currentDescription.join('\n').trim()
                };
            }

            // Start new function
            const match = line.match(/^###\s+([\w\.]+)/);
            if (match) {
                currentFunction = match[1];
                currentSignature = '';
                currentDescription = [];
                capturingDescription = false;
            } else {
                currentFunction = null;
            }
            continue;
        }

        if (currentFunction) {
            if (!currentSignature && line.startsWith('`') && line.endsWith('`')) {
                currentSignature = line.slice(1, -1);
                capturingDescription = true;
                continue;
            }

            if (capturingDescription) {
                 if (line.length > 0) {
                     currentDescription.push(line);
                 }
            }
        }
    }

    // Add the last function or library
    if (inLibrarySection && currentLibrary) {
        docs[currentLibrary] = {
            signature: currentLibrary,
            description: currentDescription.join('\n').trim()
        };
    } else if (currentFunction) {
        docs[currentFunction] = {
            signature: currentSignature,
            description: currentDescription.join('\n').trim()
        };
    }

    fs.writeFileSync(outputPath, JSON.stringify(docs, null, 2));
    console.log(`Successfully generated docs for ${Object.keys(docs).length} items.`);

} catch (error) {
    console.error('Error generating docs:', error);
    process.exit(1);
}
