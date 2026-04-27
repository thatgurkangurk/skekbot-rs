export type UtilsModule = {
    getUptime: () -> number,
    sleep: (seconds: number) -> (),
    stringify: (value: {}) -> string
}

return {} :: UtilsModule