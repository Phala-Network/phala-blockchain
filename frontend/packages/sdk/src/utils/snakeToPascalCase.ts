export function snakeToPascalCase(snakeStr: string): string {
  return snakeStr
    .toLowerCase()
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join('')
}

// a regex that detect the string is pascal case style or not.
const regex_pascal_case = /^[A-Z][a-zA-Z0-9]*$/

export function isPascalCase(str: string): boolean {
  return regex_pascal_case.test(str)
}
