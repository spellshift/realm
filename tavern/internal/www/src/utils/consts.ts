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
export type SessionType = {
    hostname: string;
    id: string;
    name: string;
    principal: string;
    tags: Array<TomeTag>;
}
export type SelectedSessions = {
    [sessionId: string]: boolean
};
export type TagContextType = {
    sessions: Array<SessionType>,
    groupTags: Array<TomeTag>,
    serviceTags: Array<TomeTag>
}
export type CreateJobProps = {
    name: string,
    tome: Tome | null,
    params: any,
    sessions: Array<string>,
};
export type Task = {
    id: string,
    lastModifiedAt: string,
    output: string,
    execStartedAt: string,
    execFinishedAt: string,
    session: SessionType
    createdAt: string,
};
export type JobProps = {
    id: string,
    name: string,
    tasks: Array<Task>,
    tome: Tome
}


