const COLOR_STRING = "\x1b[38;2;197;148;124m";
const COLOR_PUNCTUATION_1 = "\x1b[38;2;249;217;73m";
const COLOR_PUNCTUATION_2 = "\x1b[38;2;204;118;209m";
const RESET = "\x1b[0m";

const fStringRegex = /f(["'])(?:\\.|[^\\])*?\1/g;

const text = "x = f\"Hello {name} and {age + 1} and {{escaped}}\" + 'other string'";
let lastIndex = 0;
let result = "";
let match;
while ((match = fStringRegex.exec(text)) !== null) {
  if (match.index > lastIndex) {
    result += text.slice(lastIndex, match.index);
  }

  const fstr = match[0];
  const replaced = fstr.replace(/\{\{|\}\}|\{[^}]*\}/g, (m) => {
    if (m === '{{' || m === '}}') return m;
    const inner = m.slice(1, -1);
    // Recursively highlight inner, though simplified here:
    return `${RESET}${COLOR_PUNCTUATION_1}{${RESET}${inner}${COLOR_PUNCTUATION_1}}${COLOR_STRING}`;
  });
  result += `${COLOR_STRING}${replaced}${RESET}`;

  lastIndex = fStringRegex.lastIndex;
}
if (lastIndex < text.length) {
  result += text.slice(lastIndex);
}
console.log(result);
