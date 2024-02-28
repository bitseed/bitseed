import * as bitcoin from 'bitcoinjs-lib';

export interface ScriptPubKey {
    asm: string;
    desc: string;
    hex: string;
    address: string;
    type: string;
}

const classifyOutputScript = (script: Buffer): string => {
  const isOutput = (paymentFn: (params: { output?: Buffer }) => bitcoin.payments.Payment) => {
    try { 
      return paymentFn({ output: script }) !== undefined;
    } catch (e) {
      return false;
    }
  }

  if (isOutput(bitcoin.payments.p2pk)) return 'P2PK';
  else if (isOutput(bitcoin.payments.p2pkh)) return 'P2PKH';
  else if (isOutput(bitcoin.payments.p2ms)) return 'P2MS';  
  else if (isOutput(bitcoin.payments.p2wpkh)) return 'P2WPKH';
  else if (isOutput(bitcoin.payments.p2sh)) return 'P2SH';
  else if (isOutput(bitcoin.payments.p2tr)) return 'P2TR';
  
  return 'nonstandard';
}

export function decodeScriptPubKey(scriptPubKeyHex: string, network: bitcoin.Network): ScriptPubKey {
  const scriptPubKeyBuffer = Buffer.from(scriptPubKeyHex, 'hex');
  const decompiled = bitcoin.script.decompile(scriptPubKeyBuffer);
  if (!decompiled) {
      throw new Error('Invalid scriptPubKey: Unable to decompile');
  }
  const asm = bitcoin.script.toASM(decompiled);
  const type = classifyOutputScript(scriptPubKeyBuffer);

  let address: string = ""

  try {
    if (['P2PKH', 'P2PK', 'P2MS', 'P2WPKH', 'P2SH', 'P2TR'].includes(type)) {
      address = bitcoin.address.fromOutputScript(scriptPubKeyBuffer, network);
    }
  } catch (error) {
    // Log the error or handle it as needed
    console.error('Error getting address from output script:', error);
  }

  return {
      asm,
      desc: `Script ${type}`,
      hex: scriptPubKeyHex,
      address,
      type
  };
}
