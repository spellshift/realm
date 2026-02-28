const fs = require('fs');
const content = fs.readFileSync('tavern/internal/www/src/pages/shellv2/hooks/useShellTerminal.ts', 'utf8');
const hasPaste = content.includes('onPaste');
console.log('has paste:', hasPaste);
