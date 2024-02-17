import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react';
import DeploStory from './deploy.story';

test.use({ viewport: { width: 500, height: 500 } });

test('Deploy tick should be ok', async ({ page, mount  }) => {
  const component = await mount(<DeploStory />);
  await expect(component).toContainText('Deploy: bitseed');
});
