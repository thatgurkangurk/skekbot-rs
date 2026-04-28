export type ServerSettings = { id: number, hidden_chance: number, dad_enabled: boolean }

export type DbModule = {
	getOrCreateServerSettings: (server_id: string) -> ServerSettings,
}

return {} :: DbModule
