export type LogModule = {
    log: (level: string, message: string) -> (),
}

return {} :: LogModule