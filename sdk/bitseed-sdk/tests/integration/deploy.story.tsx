import React, { useEffect, useState } from "react";
import { JsonRpcDatasource } from "@sadoprotocol/ordit-sdk";
import { Ordit } from "@sadoprotocol/ordit-sdk";
import { BitSeed, BitSeedApiMock, Generator, InscriptionID, DeployOptions } from '../../src';

const MNEMONIC = "<mnemonic>";
const network = "testnet";
const datasource = new JsonRpcDatasource({ network });
const bitseedApiMock = new BitSeedApiMock();

export default function DeployStory() {
  const [bitseed, setBitseed] = useState<BitSeed | undefined>(undefined);
  const [tick, setTick] = useState<string>('');
  const [max, setMax] = useState<number>(0);
  const [generatorType, setGeneratorType] = useState<"Bytes" | "InscriptionID" | "File">("Bytes");
  const [generatorValue, setGeneratorValue] = useState<string>('');
  const [file, setFile] = useState<File | null>(null);
  const [deployResult, setDeployResult] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);

  useEffect(() => {
    const wallet = new Ordit({
      bip39: MNEMONIC,
      network
    });

    wallet.setDefaultAddress('taproot');

    const bitseed = new BitSeed(wallet, datasource, bitseedApiMock);
    setBitseed(bitseed);
  }, []);

  const handleDeploy = async () => {
    if (!bitseed) return;

    try {
      let generator: Generator;
      switch (generatorType) {
        case "Bytes":
          generator = new Uint8Array(Buffer.from(generatorValue, 'base64')); // Assuming base64 input for bytes
          break;
        case "InscriptionID":
          generator = generatorValue as InscriptionID;
          break;
        case "File":
          if (!file) throw new Error("File is required for File generator type");
          generator = await readFileAsBytes(file); // Convert file to bytes
          break;
        default:
          throw new Error("Invalid generator type");
      }

      const deployOptions: DeployOptions = {/* ... */}; // Replace with actual options if needed

      const inscriptionId = await bitseed.deploy(tick, max, generator, deployOptions);
      setDeployResult(inscriptionId);
      setError(undefined);
    } catch (e) {
      setError(e.message);
      setDeployResult(undefined);
    }
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (files && files.length > 0) {
      setFile(files[0]);
      setGeneratorType("File"); // Automatically set to File type
    } else {
      setFile(null);
    }
  };

  // 读取文件内容并转换为 Uint8Array 的函数
  const readFileAsBytes = (file: File): Promise<Uint8Array> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (event) => {
        const result = event.target?.result;
        if (result) {
          resolve(new Uint8Array(result as ArrayBuffer));
        } else {
          reject(new Error("Failed to read file"));
        }
      };
      reader.onerror = (event) => {
        reject(new Error(`FileReader error: ${event.target?.error?.message}`));
      };
      reader.readAsArrayBuffer(file);
    });
  };

  return (
    <div>
      <div>
        Deploy: {bitseed?.name()}
      </div>
      <div>
        <input type="text" placeholder="Tick" value={tick} onChange={(e) => setTick(e.target.value)} />
        <input type="number" placeholder="Max" value={max} onChange={(e) => setMax(Number(e.target.value))} />
        <select value={generatorType} onChange={(e) => setGeneratorType(e.target.value as any)}>
          <option value="Bytes">Bytes</option>
          <option value="InscriptionID">InscriptionID</option>
          <option value="File">File</option>
        </select>
        {generatorType === 'InscriptionID' && (
          <input type="text" placeholder="InscriptionID" value={generatorValue} onChange={(e) => setGeneratorValue(e.target.value)} />
        )}
        {generatorType === 'Bytes' && (
          <input type="text" placeholder="Generator Value (Base64)" value={generatorValue} onChange={(e) => setGeneratorValue(e.target.value)} />
        )}
        {generatorType === 'File' && (
          <input type="file" onChange={handleFileChange} />
        )}
        <button onClick={handleDeploy}>Deploy</button>
      </div>
      {deployResult && <div>Deploy Result: {deployResult}</div>}
      {error && <div>Error: {error}</div>}
    </div>
  );
}
