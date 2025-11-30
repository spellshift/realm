export enum StepStatus {
    Current = "CURRENT",
    Upcoming = "UPCOMING",
    Complete = "COMPLETE",
}
export enum PageNavItem {
    dashboard="Dashboard",
    quests='Quests',
    documentation='Documentation',
    playground='API Playground',
    tasks='Tasks',
    createQuest= 'Create new quest',
    hosts="Hosts",
    tomes="Tomes",
    admin="Admin",
}
export enum SupportedPlatforms {
    Windows="PLATFORM_WINDOWS",
    Linux='PLATFORM_LINUX',
    MacOS='PLATFORM_MACOS',
    BSD='PLATFORM_BSD',
    Unknown='PLATFORM_UNSPECIFIED'
}
export enum TableRowLimit {
    QuestRowLimit=8,
    TaskRowLimit=8,
    HostRowLimit=8
}
export enum PrincipalAdminTypes {
    root='root',
    Administrator='Administrator',
    SYSTEM="SYSTEM"
}

export enum TaskChartKeys {
    taskError="Tasks with errors",
    taskNoError ="Tasks without errors",
    taskCreated= "Tasks created"
}

export enum DEFAULT_QUERY_TYPE{
    hostIDQuery="HOST_ID_QUERY",
    questIdQuery= "QUEST_ID_QUERY",
    questDetailsQuery= "QUEST_DETAILS_QUERY",
}
