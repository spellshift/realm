// Resolve React to the UMD global preloaded in the artifact preview frame.
// CJS shim so esbuild permits any named import without enumerating them.
module.exports = window.React;
