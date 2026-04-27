export type Connection = {
    Disconnect: (self: Connection) -> ()
}

export type Signal<T...> = {
    Connect: (self: Signal<T...>, callback: (T...) -> ()) -> Connection
}

export type Message = {
    content: string,
    author: string,
    channel_id: string,
    guild_id: string?
}

export type EventsModule = {
    OnReady: Signal<string>,
    OnMessageCreate: Signal<Message>,
}

return {} :: EventsModule