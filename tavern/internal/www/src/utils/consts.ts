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
export type BeaconType = {
    hostname: string;
    id: string;
    name: string;
    principal: string;
    tags: Array<TomeTag>;
}
export type SelectedBeacons = {
    [beaconId: string]: boolean
};
export type TagContextType = {
    beacons: Array<BeaconType>,
    groupTags: Array<TomeTag>,
    serviceTags: Array<TomeTag>
}
export type CreateQuestProps = {
    name: string,
    tome: Tome | null,
    params: any,
    beacons: Array<string>,
};

export type Task = {
    id: string,
    lastModifiedAt: string,
    output: string,
    execStartedAt: string,
    execFinishedAt: string,
    beacon: BeaconType
    createdAt: string,
};
export type QuestProps = {
    id: string,
    name: string,
    tasks: Array<Task>,
    tome: Tome
}
