local Types = require("@skekbot/types")

export type RestModule = {
	sendMessage: (channel_id: string, content: string) -> (),
	setUserNickname: (user_id: string, guild_id: string, new_nickname: string) -> (),
	replyToMessage: (message: Types.Message, content: string) -> (),
}

return {} :: RestModule
