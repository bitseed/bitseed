export class BitseedSDKError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options)
    this.name = "BitseedSDKError"
  }
}
