// https://gist.github.com/ManUtopiK/469aec75b655d6a4d912aeb3b75af3c9#file-clean-graphql-js-L20
// Don't use yet, It seems like stripping edge/node may interfere with cache from apollo, more research is needed
type Json =
  | { [key: string]: Json }
  | Json[]
  | string
  | number
  | boolean
  | null;

type JsonObj = { [key: string]: Json };

export function transformEdgeGrapqhlObject(obj: Json): Json {
  if (isObject(obj)) {
    if (obj.edges && Array.isArray(obj.edges)) {
      return obj.edges.map((edge) => {
        if (edge && isObject(edge)) {
          return transformEdgeGrapqhlObject(edge.node);
        }

        return edge;
      });
    }

    return Object.keys(obj).reduce((result: JsonObj, key) => {
      const value = obj[key];
      result[key] = isObject(value) ? transformEdgeGrapqhlObject(value) : obj[key];
      return result;
    }, {} as JsonObj);
  }

  if (Array.isArray(obj)) {
    return obj.map(transformEdgeGrapqhlObject);
  }

  return obj;
}

function isObject(input: Json): input is JsonObj {
  return typeof input === "object" && input !== null && !Array.isArray(input);
}