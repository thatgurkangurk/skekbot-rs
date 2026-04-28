local Types = require("@skekbot/types")

export type DbModule = {
	getOrCreateServerSettings: (server_id: string) -> Types.ServerSettings,
}

return {} :: DbModule
