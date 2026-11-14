import { FilterBarOption } from "../../utils/interfacesUI";

// Only one concrete "type" today -- built agent artifacts (name prefixed
// "build/" by the builder on upload). Kept as a list so more asset types
// can be added here later without changing the filter bar itself.
export const ASSET_TYPE_OPTIONS: FilterBarOption[] = [
    {
        id: "agents",
        name: "Agents",
        value: "agents",
        label: "Agents",
        kind: "agent",
    },
];
