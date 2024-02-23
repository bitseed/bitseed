import React from 'react'

import path from 'path'
import { test, expect } from '@playwright/experimental-ct-react'
import DeployGeneratorStory from './generator.story'

test.use({ viewport: { width: 500, height: 500 } })

test('Deploy generator', async ({ mount }) => {
  const component = await mount(<DeployGeneratorStory />)

  // Upload generator wasm file
  await component
    .locator('input[placeholder="wasmFile"]')
    .setInputFiles(path.join(__dirname, '../data/generator.wasm'))

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ')
})
