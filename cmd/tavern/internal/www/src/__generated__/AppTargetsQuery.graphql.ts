/**
 * @generated SignedSource<<2e12f10a64cb447ca53aa86bcaf458ad>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest, Query } from 'relay-runtime';
export type CredentialKind = "PASSWORD" | "KEY" | "CERTIFICATE" | "%future added value";
export type AppTargetsQuery$variables = {};
export type AppTargetsQuery$data = {
  readonly targets: {
    readonly edges: ReadonlyArray<{
      readonly node: {
        readonly id: string;
        readonly name: string;
        readonly forwardConnectIP: string;
        readonly credentials: ReadonlyArray<{
          readonly id: string;
          readonly kind: CredentialKind;
          readonly principal: string;
          readonly secret: string;
        }> | null;
      } | null;
    } | null> | null;
  } | null;
};
export type AppTargetsQuery = {
  variables: AppTargetsQuery$variables;
  response: AppTargetsQuery$data;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Literal",
        "name": "where",
        "value": {
          "name": "Test"
        }
      }
    ],
    "concreteType": "TargetConnection",
    "kind": "LinkedField",
    "name": "targets",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "TargetEdge",
        "kind": "LinkedField",
        "name": "edges",
        "plural": true,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "Target",
            "kind": "LinkedField",
            "name": "node",
            "plural": false,
            "selections": [
              (v0/*: any*/),
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "name",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "forwardConnectIP",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "concreteType": "Credential",
                "kind": "LinkedField",
                "name": "credentials",
                "plural": true,
                "selections": [
                  (v0/*: any*/),
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "kind",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "principal",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "secret",
                    "storageKey": null
                  }
                ],
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "storageKey": null
      }
    ],
    "storageKey": "targets(where:{\"name\":\"Test\"})"
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "AppTargetsQuery",
    "selections": (v1/*: any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "AppTargetsQuery",
    "selections": (v1/*: any*/)
  },
  "params": {
    "cacheID": "d659caefbeb70782a33205c5dd6963d9",
    "id": null,
    "metadata": {},
    "name": "AppTargetsQuery",
    "operationKind": "query",
    "text": "query AppTargetsQuery {\n  targets(where: {name: \"Test\"}) {\n    edges {\n      node {\n        id\n        name\n        forwardConnectIP\n        credentials {\n          id\n          kind\n          principal\n          secret\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "157a5cded49d2e1e3e5e1f6616cc383a";

export default node;
