import Keyring from '@polkadot/keyring'
import { describe, expect, it, vi } from 'vitest'
import {
  PinkLoggerContractPromise,
  type SerMessageEvent,
  buildGetLogRequest,
  getTopicHash,
} from '../../src/contracts/PinkLoggerContract'
import createPruntimeClient from '../../src/pruntime/createPruntimeClient'

function buildGetLogRequestTail(...params: any[]) {
  return buildGetLogRequest(
    params,
    (x) => {
      if (!x.from) {
        return x.count ? -x.count : -10
      }
      return -(x.from + (x.count || 10))
    },
    () => ({ count: 10 })
  )
}

function buildGetLogRequestHead(params: any[]) {
  return buildGetLogRequest(
    params,
    (x) => x.from || 0,
    () => ({ from: 0, count: 10 })
  )
}

describe('buildGetLogRequest can handle variants shortcut of parameters for tail', () => {
  it('should handle 0 parameters', () => {
    expect(buildGetLogRequestTail()).toEqual({ from: -10, count: 10 })
  })

  it('should handle 1 numeric parameter fetch specified records', () => {
    expect(buildGetLogRequestTail(50)).toEqual({ from: -50, count: 50 })
    expect(buildGetLogRequestTail(2)).toEqual({ from: -2, count: 2 })
  })

  it('should handle 1 object parameter as full filter arguments', () => {
    expect(buildGetLogRequestTail({ contract: '0x123' })).toEqual({ from: -10, count: 10, contract: '0x123' })
    expect(buildGetLogRequestTail({ block_number: 100 })).toEqual({ from: -10, count: 10, block_number: 100 })
    expect(buildGetLogRequestTail({ count: 20 })).toEqual({ from: -20, count: 20 })
    expect(buildGetLogRequestTail({ contract: '0x123', block_number: 100, count: 24 })).toEqual({
      from: -24,
      count: 24,
      contract: '0x123',
      block_number: 100,
    })
  })

  it('should handle 2 parameters as count & filter', () => {
    expect(buildGetLogRequestTail(20, { contract: '0x123' })).toEqual({ from: -20, count: 20, contract: '0x123' })
    expect(buildGetLogRequestTail(20, { block_number: 100 })).toEqual({ from: -20, count: 20, block_number: 100 })
    expect(buildGetLogRequestTail(20, 40)).toEqual({ from: 40, count: 20 })
  })

  it('should handle 3 parameters as from, count & filter', () => {
    expect(buildGetLogRequestTail(20, 40, { contract: '0x123' })).toEqual({
      from: 40,
      count: 20,
      contract: '0x123',
    })
    expect(buildGetLogRequestTail(20, 40, { block_number: 100 })).toEqual({
      from: 40,
      count: 20,
      block_number: 100,
    })
    expect(buildGetLogRequestTail(20, 40, 60)).toEqual({ from: 40, count: 20 })
  })
})

describe('buildGetLogRequest can handle variants shortcut of parameters for head', () => {
  it('should handle 0 parameters', () => {
    expect(buildGetLogRequestHead([])).toEqual({ from: 0, count: 10 })
  })

  it('should handle 1 numeric parameter fetch specified records', () => {
    expect(buildGetLogRequestHead([50])).toEqual({ from: 0, count: 50 })
    expect(buildGetLogRequestHead([2])).toEqual({ from: 0, count: 2 })
  })

  it('should handle 1 object parameter as full filter arguments', () => {
    expect(buildGetLogRequestHead([{ contract: '0x123' }])).toEqual({ from: 0, count: 10, contract: '0x123' })
    expect(buildGetLogRequestHead([{ block_number: 100 }])).toEqual({ from: 0, count: 10, block_number: 100 })
    expect(buildGetLogRequestHead([{ count: 20 }])).toEqual({ from: 0, count: 20 })
    expect(buildGetLogRequestHead([{ contract: '0x123', block_number: 100, count: 24 }])).toEqual({
      from: 0,
      count: 24,
      contract: '0x123',
      block_number: 100,
    })
  })

  it('should handle 2 parameters as count & filter', () => {
    expect(buildGetLogRequestHead([20, { contract: '0x123' }])).toEqual({ from: 0, count: 20, contract: '0x123' })
    expect(buildGetLogRequestHead([20, { block_number: 100 }])).toEqual({ from: 0, count: 20, block_number: 100 })
    expect(buildGetLogRequestHead([20, 40])).toEqual({ from: 40, count: 20 })
  })

  it('should handle 3 parameters as from, count & filter', () => {
    expect(buildGetLogRequestHead([20, 40, { contract: '0x123' }])).toEqual({
      from: 40,
      count: 20,
      contract: '0x123',
    })
    expect(buildGetLogRequestHead([20, 40, { block_number: 100 }])).toEqual({
      from: 40,
      count: 20,
      block_number: 100,
    })
    expect(buildGetLogRequestHead([20, 40, 60])).toEqual({ from: 40, count: 20 })
  })
})

describe('getTopicHash should convert literal topic to hash', () => {
  it('should works as expected', () => {
    const shortTopic = getTopicHash('System::Log')
    expect(shortTopic).toEqual('0x0053797374656d3a3a4c6f670000000000000000000000000000000000000000')
    const longTopic = getTopicHash('System::ALongEventThatCouldNotBeStoredInTheTopic')
    expect(longTopic).toEqual('0xc195a2d447cc68a2902771a7242347233b1665662a47d52ac760922671dc1fb4')
  })

  it('should throw error if topic is not valid', () => {
    // @ts-ignore
    expect(() => getTopicHash('system')).toThrow()
  })
})

describe('the client-side filter type & topic should works', () => {
  it('the type filter should works', async ({ expect }) => {
    const keyring = new Keyring()
    const logger = new PinkLoggerContractPromise(
      createPruntimeClient('http://localhost:8080'),
      '',
      keyring.addFromUri('//Alice'),
      ''
    )
    // @ts-ignore
    vi.spyOn(logger, 'getLogRaw').mockImplementation(() => {
      return Promise.resolve({
        records: [
          {
            sequence: 6625,
            type: 'Log',
            blockNumber: 1821442,
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            entry: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            execMode: 'estimating',
            timestamp: 1701717261053,
            level: 1,
            message: 'contract call reverted',
          },
          {
            sequence: 6626,
            type: 'QueryIn',
            user: '0xe11964fe773024fb',
          },
          {
            sequence: 6627,
            type: 'QueryIn',
            user: '0x6aefdee5248bcfaf',
          },
          {
            sequence: 6628,
            type: 'QueryIn',
            user: '0x6aefdee5248bcfaf',
          },
          {
            sequence: 6629,
            type: 'MessageOutput',
            blockNumber: 1821449,
            origin: '0x4613e7b5fdf418fe5472071e81dcf34195805ffd2f7f68a97f7710ff7e3c3a46',
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            nonce: '0x517492b3a5e0497281a56c915a0e31c329faf93614986ca996162b31282c187f',
            output: '0x03d905cbed3a130600070000807e120200800000ff3f597307000000000000000000000000000000000008000000',
          },
          {
            sequence: 6630,
            type: 'Event',
            blockNumber: 1821449,
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            topics: ['0x2397e37a88ffc850c2bbc863df83cbb459c20fb30c190667330e625531370491'],
            payload: '0x02d41f763cb3326a5dc04f09c6eeab35ca809c756ea7e8c3fb1a9a89fc4d41c03d',
          },
        ],
        next: 0,
      })
    })

    const result = await logger.tail({ type: 'Event' })
    expect(result.records.length).toEqual(1)
    expect(result.records[0].type).toEqual('Event')
  })

  it('the topic filter should works', async ({ expect }) => {
    const keyring = new Keyring()
    const logger = new PinkLoggerContractPromise(
      createPruntimeClient('http://localhost:8080'),
      '',
      keyring.addFromUri('//Alice'),
      ''
    )
    // @ts-ignore
    vi.spyOn(logger, 'getLogRaw').mockImplementation(() => {
      return Promise.resolve({
        records: [
          {
            sequence: 6597,
            type: 'Event',
            blockNumber: 1821016,
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            topics: ['0xcf4b5a8f1631c6ee65f065cb4b426df487b9aec41615da6799315093117a7a7b'],
            payload: '0x01edab8c49dcab658c859a89f9edba627d80b22c95e48e4b6c352041e71d002223',
          },
          {
            sequence: 6598,
            type: 'Event',
            blockNumber: 1821016,
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            topics: ['0xcf4b5a8f1631c6ee65f065cb4b426df487b9aec41615da6799315093117a7a7b'],
            payload: '0x01d41f763cb3326a5dc04f09c6eeab35ca809c756ea7e8c3fb1a9a89fc4d41c03d',
          },
          {
            sequence: 6625,
            type: 'Log',
            blockNumber: 1821442,
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            entry: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            execMode: 'estimating',
            timestamp: 1701717261053,
            level: 1,
            message: 'contract call reverted',
          },
          {
            sequence: 6626,
            type: 'QueryIn',
            user: '0xe11964fe773024fb',
          },
          {
            sequence: 6627,
            type: 'QueryIn',
            user: '0x6aefdee5248bcfaf',
          },
          {
            sequence: 6628,
            type: 'QueryIn',
            user: '0x6aefdee5248bcfaf',
          },
          {
            sequence: 6629,
            type: 'MessageOutput',
            blockNumber: 1821449,
            origin: '0x4613e7b5fdf418fe5472071e81dcf34195805ffd2f7f68a97f7710ff7e3c3a46',
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            nonce: '0x517492b3a5e0497281a56c915a0e31c329faf93614986ca996162b31282c187f',
            output: '0x03d905cbed3a130600070000807e120200800000ff3f597307000000000000000000000000000000000008000000',
          },
          {
            sequence: 6630,
            type: 'Event',
            blockNumber: 1821449,
            contract: '0xc5dfcc7fff77f29ac5194d7280bfe6f6f0fb4978af7e209ef89eef590d483537',
            topics: ['0x2397e37a88ffc850c2bbc863df83cbb459c20fb30c190667330e625531370491'],
            payload: '0x02d41f763cb3326a5dc04f09c6eeab35ca809c756ea7e8c3fb1a9a89fc4d41c03d',
          },
        ],
        next: 0,
      })
    })

    const result = await logger.tail({ type: 'Event', topic: 'PhalaFaucet::QuestScriptRegistered' })
    expect(result.records.length).toEqual(2)
    expect((result.records[0] as SerMessageEvent).topics[0]).toEqual(
      '0xcf4b5a8f1631c6ee65f065cb4b426df487b9aec41615da6799315093117a7a7b'
    )
    expect((result.records[1] as SerMessageEvent).topics[0]).toEqual(
      '0xcf4b5a8f1631c6ee65f065cb4b426df487b9aec41615da6799315093117a7a7b'
    )
  })
})
