// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module.exports = {
  plugins: ['unused-imports', 'prettier', 'header', 'require-extensions'],
  extends: ['react-app', 'prettier', 'plugin:prettier/recommended', 'plugin:import/typescript'],
  settings: {
    react: {
      version: '18',
    },
    'import/resolver': {
      typescript: true,
    },
  },
  env: {
    es2020: true,
  },
  root: true,
  ignorePatterns: [
    '*.js',
    'node_modules',
    'build',
    'templates',
    'docs',
    'out',
    'generated',
    'templates',
    'dist',
    'coverage',
  ],
  rules: {
    'no-case-declarations': 'off',
    'no-implicit-coercion': [2, { number: true, string: true, boolean: false }],
    '@typescript-eslint/no-redeclare': 'off',
    '@typescript-eslint/ban-types': [
      'error',
      {
        types: {
          Buffer: 'Buffer usage increases bundle size and is not consistently implemented on web.',
        },
        extendDefaults: true,
      },
    ],
    'no-restricted-globals': [
      'error',
      {
        name: 'Buffer',
        message: 'Buffer usage increases bundle size and is not consistently implemented on web.',
      },
    ],
    '@typescript-eslint/no-unused-vars': [
      'error',
      {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        vars: 'all',
        args: 'none',
        ignoreRestSiblings: true,
      },
    ],
  },
}
