local Types = require("@skekbot/types")

export type RestModule = {
	sendMessage: (channel_id: string, content: string) -> (),
	replyToMessage: (message: Types.Message, content: string) -> (),
}

return {} :: RestModule
