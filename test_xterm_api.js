const { Terminal } = require("./tavern/internal/www/node_modules/@xterm/xterm");
const term = new Terminal();
console.log("term.onPaste exists:", !!term.onPaste);
console.log("term.attachCustomKeyEventHandler exists:", !!term.attachCustomKeyEventHandler);
