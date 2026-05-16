export type EnvModule = {
	getEnvironmentVariable: (key: string) -> string?,
}

return {} :: EnvModule
