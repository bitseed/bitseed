export type SFTContent = {
  content_type: string,
  body: Uint8Array
}

export type SFTRecord = {
  op: string,
  tick: string
  amount: number
  attributes?: Map<string, string>,
  content?: SFTContent
}
