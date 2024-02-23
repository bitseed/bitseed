import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'
import DeployStory from './deploy.story'

test.use({ viewport: { width: 500, height: 500 } })

test('Deploy tick with generator inscription_id', async ({ mount }) => {
  const component = await mount(<DeployStory />)

  const inscriptionId = 'dd1f515b828eedabd6b0be147cf611ca08c20f39058feee9b96efaa2eba43d9di0'

  // Input the InscriptionID
  await component.locator('input[placeholder="Tick"]').fill('move')
  await component.locator('input[placeholder="Max"]').fill('1000')
  await component.locator('input[placeholder="InscriptionID"]').fill(inscriptionId)

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ')
})
