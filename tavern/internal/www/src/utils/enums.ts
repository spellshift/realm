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
    quests='Quest history',
    documentation='Documentation',
    playground='API Playground',
    results='Quest outputs',
    createQuest= 'Create new quest',
    hosts="Hosts"
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
