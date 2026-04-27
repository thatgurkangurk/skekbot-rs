-- represents an active event connection
export type Connection = {
    Disconnect: (self: Connection) -> ()
}

-- represents a roblox-style event signal
export type Signal<T...> = {
    Connect: (self: Signal<T...>, callback: (T...) -> ()) -> Connection
}

-- represents the serenity message payload
export type Message = {
    content: string,
    author: string,
    channel_id: string,
    guild_id: string?
}

export type ServerSettings = {
    id: string,
    hidden_chance: number,
    dad_enabled: boolean
}

export type Skekbot = {
    Events: {
        OnReady: Signal<string>,
        OnMessageCreate: Signal<Message>,
    },

    Rest: {
        sendMessage: (channel_id: string, content: string) -> (),
    },

    Db: {
        getOrCreateServerSettings: (guild_id: string) -> ServerSettings
    },

    log: (level: string, message: string) -> (),
    getUptime: () -> number,

    sleep: (seconds: number) -> (),
}

declare Skekbot: Skekbot