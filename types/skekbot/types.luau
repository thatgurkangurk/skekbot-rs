export type ServerSettings = { id: number, hidden_chance: number, dad_enabled: boolean }
export type Message = {
	id: string,
	content: string,
	author: { id: string, bot: boolean, username: string, global_name: string? },
	channel_id: string,
	guild_id: string?,
}

export type TypesModule = {}

return {} :: TypesModule
