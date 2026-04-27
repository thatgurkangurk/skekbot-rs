export type ServerSettings = {
    id: string,
    hidden_chance: number,
    dad_enabled: boolean
}

export type DbModule = {
    getOrCreateServerSettings: (guild_id: string) -> ServerSettings
}

return {} :: DbModule