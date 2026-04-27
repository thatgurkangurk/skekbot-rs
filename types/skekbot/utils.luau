export type UtilsModule = {
    getUptime: () -> number,
    sleep: (seconds: number) -> (),
}

return {} :: UtilsModule