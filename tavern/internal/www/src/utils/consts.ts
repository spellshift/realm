import { string } from "yup";
import { SupportedPlatforms } from "./enums";

export type FormStep = {
    name: string;
    description: string;
    href: string;
    step: number;
}
export type Tome = {
    description: string;
    eldritch: string;
    id: string;
    name: string;
    paramDefs: string;
    tactic: string;
    supportModel: string;

}
export type TomeParams = {
    name: string;
    label: string;
    type: string;
    placeholder: string;
    value?: any;
}
export type TomeTag = {
    id: string;
    name: string;
    kind: string;
}
export type TagOptionType = {
    value: string,
    label: string,
} & TomeTag;

export type KindOfTag = 'service' | 'group';

export type FilterBarOption = {
    label?: string;
    id: string;
    name: string;
    kind: string;
}
export type HostType = {
    id: string;
    name: string;
    primaryIP?: string;
    lastSeenAt?: string | null;
    platform?: SupportedPlatforms;
    tags?: Array<TomeTag>;
    beacons?: Array<BeaconType>;
    credentials?: Array<CredentialType>;
}
export type BeaconType = {
    id: string;
    name: string;
    principal: string;
    host: HostType;
    lastSeenAt: string;
    interval: number;
}
export type SelectedBeacons = {
    [beaconId: string]: boolean
};
export type TagContextType = {
    beacons: Array<BeaconType>;
    groupTags: Array<TomeTag>;
    serviceTags: Array<TomeTag>;
    hosts: Array<HostType>;
}
export type QuestParam = {
    label: string,
    name: string,
    placeholder: string,
    type: string,
    value: string,
}
export type CreateQuestProps = {
    name: string,
    tome: Tome | null,
    params: Array<QuestParam>,
    beacons: Array<string>,
};
export type Shell = {
    id: string,
    closedAt: string,
    activeUsers: Array<UserType>
}
export type Task = {
    id: string,
    lastModifiedAt: string,
    outputSize: number,
    output?: string,
    execStartedAt: string,
    execFinishedAt: string,
    beacon: BeaconType,
    createdAt: string,
    error: string,
    quest?: QuestProps,
    shells: Array<Shell>
};
export type UserType = {
    id: string;
    name: string;
    photoURL?: string | null;
    isActivated?: boolean;
    isAdmin?: boolean;
}
export type QuestProps = {
    id: string,
    name: string,
    tasks: Array<Task>,
    tome: Tome,
    creator: UserType,
    parameters: string
}
export type OutputTableProps = {
    quest: string,
    creator: UserType,
    tome: string,
    beacon: string,
    service: string | null,
    group: string | null,
    output: string,
    taskDetails?: Task

}

export type RepositoryRow = {
    node: RepositoryType
}
export type RepositoryType = {
    id?: string;
    url: string;
    tomes: Array<Tome>;
    owner?: UserType;
    repoType?: string;
    lastModifiedA?: string;
    publicKey?: string;
}

export type QuestTableRowType = {
    node: {
        id: string;
        name: string;
        tome: string;
        creator: UserType;
        lastUpdatedTask: {
            edges: {
                node: {
                    lastModifiedAt: string
                }
            }
        }
        tasksCount: {
            totalCount: number
        };
        tasksFinished: {
            totalCount: number
        };
        tasksOutput: {
            totalCount: number
        };
        tasksError: {
            totalCount: number
        };
        lastModifiedAt: null | string,
    }
}

export type PaginationPageInfo = {
    hasNextPage: boolean;
    hasPreviousPage: boolean;
    startCursor: string;
    endCursor: string;
}

export type UpdateUserProps = {
    id: number,
    activated: boolean,
    admin: boolean,
};

export type CredentialType = {
    principal: string;
    kind: string;
    secret: string;
    createdAt: string;
    lastModifiedAt: string;
}

export type NavigationItemType = {
    name: string;
    href: string;
    icon?: any;
    target?: string;
    internal?: boolean;
    adminOnly?: boolean;
}
