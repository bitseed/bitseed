import React from 'react'
import { DocsThemeConfig } from 'nextra-theme-docs'

const config: DocsThemeConfig = {
  logo: <span>Bitseed</span>,
  project: {
    link: 'https://github.com/bitseed/bitseed',
  },
  chat: {
    link: 'https://t.me/bitseed_protocol',
  },
  docsRepositoryBase: 'https://github.com/bitseed/bitseed/docs',
  i18n: [
    { locale: 'en-US', text: 'English' },
    { locale: 'zh-CN', text: '中文' }
  ],
  footer: {
    text: 'Bitseed Protocol',
  }, 
}

export default config
