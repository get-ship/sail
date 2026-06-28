import { controller, useForm } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

interface SignupFields { name: string; email: string; accept: boolean }

export default controller<Scope>({
  scope: () => ({}),
  use: {
    form: useForm<SignupFields>({
      initial: { name: '', email: '', accept: false },
      validate: ({ name, email, accept }) => ({
        name:   name.trim() === ''    ? 'required'                : null,
        email:  !email.includes('@')  ? 'must include @'          : null,
        accept: !accept               ? 'you must accept terms'   : null,
      }),
      submit: async (values) => {
        // Simulated submit — 70% success rate.
        await new Promise((r) => setTimeout(r, 800))
        if (Math.random() < 0.3) throw new Error('network is angry today')
        console.log('signed up:', values)
      },
    }),
  },
})
