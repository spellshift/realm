import { highlightPythonSyntax } from "./tavern/internal/www/src/pages/shellv2/hooks/shellUtils";
console.log(highlightPythonSyntax('x = f"Hello {name} {age + 1} and {{escaped}}"'));
