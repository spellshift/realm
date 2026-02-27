const fs = require('fs');

const path = 'tavern/internal/www/src/pages/shellv2/hooks/shellUtils.ts';
let content = fs.readFileSync(path, 'utf8');

// The issue is that the builtins are joined to form a regex: `(\b(?:sys|sys\.shell|...)\b)`
// And because it's a pipe without sorting by length descending, `sys` matches before `sys.shell`!
// We need to sort `builtins` by length descending before joining.

content = content.replace(
    'const builtins = Object.keys(docsData).map(k => k.replace(/\\./g, "\\\\."));',
    'const builtins = Object.keys(docsData).sort((a, b) => b.length - a.length).map(k => k.replace(/\\./g, "\\\\."));'
);

fs.writeFileSync(path, content);
