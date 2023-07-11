export default function assert(expected: boolean, message: string): void {
  if (!expected) {
    throw new Error(message)
  }
}
