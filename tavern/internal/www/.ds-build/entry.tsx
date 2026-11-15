// Bundle entry: expose every design-system export plus PreviewShell on
// window.Realm (the namespace the artifact previews mount from).
import * as DS from "../src/design-system";
import { gql } from "@apollo/client";
import { Tab } from "@headlessui/react";
import { PreviewShell } from "./preview-shell";

const R: Record<string, any> = {};
for (const k of Object.keys(DS)) R[k] = (DS as any)[k];
R.PreviewShell = PreviewShell;
// Non-component helper (invisible in the Components view) so low-level
// primitive previews (VirtualizedTable/CardList) can build a query document.
R.__gql = gql;
R.__Tab = Tab; // headlessui Tab group, for tab-strip previews that expect a parent Tab.Group
(window as any).Realm = R;
