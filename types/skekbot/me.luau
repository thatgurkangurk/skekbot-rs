local Types = require("@skekbot/types")

export type MeModule = {
	getCurrentUser: () -> Types.User,
}

return {} :: MeModule
