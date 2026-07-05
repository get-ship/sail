import { variants } from './variants'
type ButtonProps = { variant: 'solid' | 'outline'; size: 'sm' | 'lg' }
export const bad = variants<ButtonProps>('button', {
  variants: {
    variant: { solidd: 'bg:primary' },
    size: { sm: 42 },
  },
  compound: [{ when: { variant: 'ghost' } }],
})
