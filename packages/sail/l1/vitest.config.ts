import { sailComponentVitestConfig } from '@ship/test-setup/vitest'

export default sailComponentVitestConfig({
  environment: 'happy-dom',
  include: ['test/**/*.test.ts'],
})
