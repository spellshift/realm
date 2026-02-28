const COLOR_STRING = "\x1b[38;2;197;148;124m";
const COLOR_PUNCTUATION_1 = "\x1b[38;2;249;217;73m";
const RESET = "\x1b[0m";

const regex = new RegExp([
    // 1. f-Strings (single/double quoted)
    /(f(?:(["'])(?:\\.|[^\\])*?\2))/.source,
    // 2. Normal Strings (single/double quoted)
    /((["'])(?:\\.|[^\\])*?\2)/.source,
].join("|"), "g");

const input = 'f"Hello {name} {age + 1}" + "other string"';
let match;
while ((match = regex.exec(input)) !== null) {
  console.log("Matched:", match[0], "f-string:", match[1], "string:", match[3]);
}
