import { InscriptionID } from '../types'

export function parseInscriptionID(id: string): InscriptionID {
  // Regular expression to match the hexadecimal txid and the index
  const match = id.match(/([a-fA-F0-9]+)(i)(\d+)$/);
  if (!match) {
    throw new Error('Invalid InscriptionID format');
  }

  // Extract the txid and index from the matched groups
  const txid = match[1];
  const index = parseInt(match[3], 10);

  return { txid, index };
}

export function inscriptionIDToString(inscriptionID: InscriptionID): string {
  return `${inscriptionID.txid}i${inscriptionID.index}`;
}
