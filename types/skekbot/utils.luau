export type UtilsModule = {
	getUptime: () -> number,
	stringify: (value: {}) -> string,
	sanitisePings: (message: string) -> string,
}

return {} :: UtilsModule
