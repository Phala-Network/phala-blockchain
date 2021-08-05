// U64F64

const BN = require('bn.js');
const { Decimal } = require('decimal.js');

class FixedPointConverter {
    constructor(nbits = 128, fracBits = 64) {
        if (nbits < fracBits || nbits < 0 || fracBits < 0) {
            throw new Error('No');
        }
        this.nbits = nbits;
        this.fracBits = fracBits;
        this.baseDec = new Decimal(2).pow(fracBits);
        // this.boundryDec = new Decimal(2).pow(nbits);
        this.boundryBn = new BN(2).pow(new BN(nbits));
    }

    // Converts U64F64 bits (in bn.js) to decimals
    fromBits(bits) {
        this.checkOverflow(bits);
        const raw = new Decimal(bits.toString());
        return raw.div(this.baseDec);
    }

    // Converts decimals to U64F64 bits
    toBits(d) {
        const raw = this.baseDec.mul(d).round();
        const bits = new BN(raw.toFixed());
        this.checkOverflow(bits);
        return bits;
    }

    checkOverflow(bits) {
        if (bits.gte(this.boundryBn)) {
            throw new Error(`Overflow: bits:${strBits} >= boundry:${strBoundry}`);
        }
    }
}

module.exports = { FixedPointConverter };

function testConverter() {
    const fpc = new FixedPointConverter(8, 4);
    for (let i = 0; i < 256; i++) {
        const bits = new BN(i);
        const d = fpc.fromBits(bits);
        const backBits = fpc.toBits(d);
        console.assert(bits.toString() == backBits.toString(), 'Bits not match');
        console.log(`${i},${d.toString()}`);
    }
}
