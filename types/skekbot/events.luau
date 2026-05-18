local Types = require("@skekbot/types")

export type Connection = { Disconnect: (self: Connection) -> () }
export type Signal<T> = { Connect: (self: Signal<T>, callback: (T) -> ()) -> () }

export type EventsModule = {
	OnReady: Signal<string>,
	OnMessageCreate: Signal<Types.Message>,
	OnGuildMemberUpdate: Signal<Types.GuildMemberUpdate>,
}

return {} :: EventsModule
