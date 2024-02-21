import React, { useEffect, useState } from "react";

import { BitSeed, InscriptionID, parseInscriptionID, inscriptionIDToString, DeployOptions } from '../../src'
import { createTestBitSeed } from './commons/test_bitseed'

export default function DeployStory() {
  const [bitseed, setBitseed] = useState<BitSeed | undefined>(undefined);
  const [tick, setTick] = useState<string>('');
  const [max, setMax] = useState<number>(0);
  const [generatorValue, setGeneratorValue] = useState<string>('');
  const [deployResult, setDeployResult] = useState<InscriptionID | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);

  useEffect(() => {
    setBitseed(createTestBitSeed());
  }, []);

  const handleDeploy = async () => {
    if (!bitseed) return;

    console.log("handle deploy start")

    try {
      let generator = parseInscriptionID(generatorValue);

      const deployOptions: DeployOptions = {
        fee_rate: 1,
      };

      const inscriptionId = await bitseed.deploy(tick, max, generator, deployOptions);
      setDeployResult(inscriptionId);
      setError(undefined);
    } catch (e) {
      console.log("deploy bitseed error:", e)
      setError(e.message);
      setDeployResult(undefined);
    }
  };

  return (
    <div>
      <div>
        Deploy Tick
      </div>
      <div>
        <input type="text" placeholder="Tick" value={tick} onChange={(e) => setTick(e.target.value)} />
        <input type="number" placeholder="Max" value={max} onChange={(e) => setMax(Number(e.target.value))} />
        <input type="text" placeholder="InscriptionID" value={generatorValue} onChange={(e) => setGeneratorValue(e.target.value)} />
        <button onClick={handleDeploy}>Deploy</button>
      </div>
      {deployResult && <div>Deploy Result: {inscriptionIDToString(deployResult)}</div>}
      {error && <div>Error: {error}</div>}
    </div>
  );
}
