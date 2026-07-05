import { variants } from './variants'
type ButtonProps = { variant: 'solid' | 'outline' | 'default'; size: 'sm' | 'lg' }
export const buttonVariants = variants<ButtonProps>('button', {
  variants: {
    variant: {
      solid:   'bg:primary fg:background',
      outline: 'border:strong fg:foreground',
      default: 'bg:background border:border fg:foreground',
    },
    size: { sm: 'min-h:control-h-sm px:2', lg: 'min-h:control-h-lg px:4' },
  },
  compound: [{ when: { variant: 'solid', size: 'lg' }, sx: 'shadow:md' }],
})
