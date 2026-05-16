export type HiddenModule = {
	warmup: () -> (),
	getHiddenNoun: (message: string) -> string?,
}

return {} :: HiddenModule
