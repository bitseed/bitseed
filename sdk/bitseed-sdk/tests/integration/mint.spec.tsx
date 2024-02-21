import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react';
import MintStory from './mint.story';

test.use({ viewport: { width: 500, height: 500 } });

test('mint tick', async ({ mount }) => {
  const component = await mount(<MintStory />);
  
  const moveTickInscriptionId = 'dd1f515b828eedabd6b0be147cf611ca08c20f39058feee9b96efaa2eba43d9di0';

  // Input the InscriptionID
  await component.locator('input[placeholder="TickDeployID"]').fill(moveTickInscriptionId);
  await component.locator('input[placeholder="UserInput"]').fill('xxxx');

  // Click the mint button
  await component.locator('button:has-text("Mint")').click();

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText("Mint Result: ");
});
