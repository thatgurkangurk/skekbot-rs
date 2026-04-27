
export type ServerSettings = {
    guild_id: string,
    prefix: string,
    logging_channel: string?,
}
    

export type DbModule = {
    getOrCreateServerSettings: (server_id: string) -> ServerSettings,
}

return {} :: DbModule
