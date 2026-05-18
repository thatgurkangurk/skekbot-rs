export type ServerSettings = { id: number, hidden_chance: number, dad_enabled: boolean }
export type User = { id: string, bot: boolean, username: string, global_name: string? }
export type Message = { id: string, content: string, author: User, channel_id: string, guild_id: string? }
export type GuildMemberUpdate = { guild_id: string, user: User, nick: string? }

export type TypesModule = {}

return {} :: TypesModule
