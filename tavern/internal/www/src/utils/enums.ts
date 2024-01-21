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
    Windows="Windows",
    Linux='Linux',
    MacOS='MacOS',
    BSD='BSD',
    Unknown='Unknown'
}
export enum TableRowLimit {
    TaskRowLimit=8
}
export enum PrincipalAdminTypes {
    root='root',
    Administrator='Administrator',
    SYSTEM="SYSTEM"
}
