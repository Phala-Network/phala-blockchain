import { describe, it } from 'vitest'
import { snakeToPascalCase } from '../../src/utils/snakeToPascalCase'

describe('snake to pascal case', () => {
  it('should convert snake case to pascal case', ({ expect }) => {
    const snakeCaseString = 'this_is_a_snake_case_string'
    const upperCamelCaseString = snakeToPascalCase(snakeCaseString)
    expect(upperCamelCaseString).toBe('ThisIsASnakeCaseString')
  })

  it('should convert snake case to pascal case and case insentive', ({ expect }) => {
    const snakeCaseString = 'This_is_a_snaKe_cAse_string'
    const upperCamelCaseString = snakeToPascalCase(snakeCaseString)
    expect(upperCamelCaseString).toBe('ThisIsASnakeCaseString')
  })
})
