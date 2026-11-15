// Automatic JSX runtime implemented over the UMD React global.
var React = window.React;
function jsx(type, props, key) {
  var p = props || {};
  var children = p.children;
  var rest = {};
  for (var k in p) { if (k !== "children") rest[k] = p[k]; }
  if (key !== undefined) rest.key = key;
  if (children === undefined) return React.createElement(type, rest);
  if (Array.isArray(children)) return React.createElement.apply(null, [type, rest].concat(children));
  return React.createElement(type, rest, children);
}
module.exports = { jsx: jsx, jsxs: jsx, jsxDEV: jsx, Fragment: React.Fragment };
