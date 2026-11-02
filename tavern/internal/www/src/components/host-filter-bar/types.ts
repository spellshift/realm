import { HostFilterRow } from "../../utils/interfacesUI";
import { HostFilterFieldDef } from "./constants";

export interface HostFilterBarProps {
    value: HostFilterRow[];
    onChange: (rows: HostFilterRow[]) => void;
    title?: string;
    fields?: HostFilterFieldDef[];
}
