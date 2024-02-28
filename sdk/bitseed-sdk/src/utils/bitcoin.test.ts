import * as bitcoin from 'bitcoinjs-lib';
import { decodeScriptPubKey } from './bitcoin';

describe('decodeScriptPubKey', () => {
  const network = bitcoin.networks.testnet;

  it('should decode a valid P2WPKH scriptPubKey', () => {
    const scriptPubKeyHex = '00148fcd888b6817682f90cdbbd7f795316c61f6da65';
    const result = decodeScriptPubKey(scriptPubKeyHex, network);

    expect(result.asm).toContain('OP_0 8fcd888b6817682f90cdbbd7f795316c61f6da65');
    expect(result.address).toBe('tb1q3lxc3zmgza5zlyxdh0tl09f3d3sldkn9ftwu3c')
    expect(result.type).toBe('P2WPKH');
  });

  it('should decode a valid P2TR scriptPubKey', () => {
    const scriptPubKeyHex = '5120114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095';
    const result = decodeScriptPubKey(scriptPubKeyHex, network);

    expect(result.asm).toContain('OP_1 114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095');
    expect(result.address).toBe('')
    expect(result.type).toBe('nonstandard');
  });

  it('should throw an error for an invalid scriptPubKey', () => {
    const invalidScriptPubKeyHex = '002a1d9';

    expect(() => {
      decodeScriptPubKey(invalidScriptPubKeyHex, network);
    }).toThrow();
  });

  it('should return nonstandard for an unknown script type', () => {
    const nonStandardScriptPubKeyHex = '6a';

    const result = decodeScriptPubKey(nonStandardScriptPubKeyHex, network);

    expect(result.type).toBe('nonstandard');
    expect(result.address).toBe('')
  });
});
