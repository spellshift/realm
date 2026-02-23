import { Option } from "../../components/tavern-base-ui/SingleDropdownSelector";
import { AssetOrderField, HostOrderField, OrderDirection, PageNavItem, ProcessOrderField, QuestOrderField, TaskOrderField } from "../../utils/enums";
import { OrderByField } from "../../utils/interfacesQuery";

export const sortablePageNavItems = [
    PageNavItem.hosts,
    PageNavItem.quests,
    PageNavItem.tasks,
    PageNavItem.assets,
    PageNavItem.processes,
] as const;

export type SortablePageNavItem = typeof sortablePageNavItems[number];

export type Sorts = Record<SortablePageNavItem, OrderByField>

type CreateOrdeFieldOptions = QuestOrderField | TaskOrderField | HostOrderField | AssetOrderField | ProcessOrderField;

export function formatEnumLabel(enumValue: string): string {
    return enumValue
        .split('_')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
        .join(' ');
}

function createOrderFieldOptions<T extends CreateOrdeFieldOptions>(
    enumObj: Record<string, T>
): Array<Option> {
    return Object.values(enumObj).map((field) => ({
        label: formatEnumLabel(field),
        value: field,
    }));
}

export const orderFieldOptionsMap = {
    [PageNavItem.hosts]: createOrderFieldOptions(HostOrderField),
    [PageNavItem.quests]: createOrderFieldOptions(QuestOrderField),
    [PageNavItem.tasks]: createOrderFieldOptions(TaskOrderField),
    [PageNavItem.assets]: createOrderFieldOptions(AssetOrderField),
    [PageNavItem.processes]: createOrderFieldOptions(ProcessOrderField),
};

export const defaultSorts: Sorts = {
    [PageNavItem.hosts]: {
        direction: OrderDirection.Desc,
        field: HostOrderField.CreatedAt
    },
    [PageNavItem.quests]: {
        direction: OrderDirection.Desc,
        field: QuestOrderField.CreatedAt
    },
    [PageNavItem.tasks]: {
        direction: OrderDirection.Desc,
        field: TaskOrderField.LastModifiedAt
    },
    [PageNavItem.assets]: {
        direction: OrderDirection.Desc,
        field: AssetOrderField.CreatedAt
    },
    [PageNavItem.processes]: {
        direction: OrderDirection.Desc,
        field: ProcessOrderField.LastModifiedAt
    }
}