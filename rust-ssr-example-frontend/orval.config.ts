import { defineConfig } from 'orval';

export default defineConfig({
  stockHistory: {
    input: 'docs/stock-history.yml',
    output: {
      mode: 'tags-split',
      target: 'lib/__generated__/stock-history.ts',
      schemas: 'lib/__generated__/models',
      client: 'react-query',
      httpClient: 'fetch',
      baseUrl: process.env.NEXT_PUBLIC_API_BASE_URL ?? 'http://localhost:8000',
      mock: {
        generators: [{ type: 'msw' }],
      },
    },
  },
});
