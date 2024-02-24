export enum StepStatus {
    Current = "CURRENT",
    Upcoming = "UPCOMING",
    Complete = "COMPLETE",
}
export enum TaskStatus {
    inprogress = "IN-PROGRESS",
    finished = "FINISHED",
    queued = "QUEUED",
}
export enum PageNavItem {
    dashboard="Dashboard",
    quests='Quest history',
    documentation='Documentation',
    playground='API Playground',
    tasks='Quest tasks',
    createQuest= 'Create new quest',
    hosts="Hosts",
    tomes="Tomes"
}
export enum SupportedPlatforms {
    Windows="PLATFORM_WINDOWS",
    Linux='PLATFORM_LINUX',
    MacOS='PLATFORM_MACOS',
    BSD='PLATFORM_BSD',
    Unknown='PLATFORM_UNSPECIFIED'
}
export enum TableRowLimit {
    TaskRowLimit=8
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
