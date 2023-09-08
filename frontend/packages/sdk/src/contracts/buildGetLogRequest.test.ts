import { describe, it, expect } from 'vitest'

import { buildGetLogRequest } from './PinkLoggerContract'

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
    return buildGetLogRequest(params, (x) => x.from || 0, () => ({ from: 0, count: 10 }))
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
