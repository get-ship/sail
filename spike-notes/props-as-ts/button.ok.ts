import { props, tone, size, bool, string } from './props'
export const buttonProps = props({
  tone,                                              // built-in: 8 tones + tone contract, free
  size,                                              // built-in: sm|md|lg + size contract
  variant: ['solid', 'soft', 'outline', 'ghost', 'default'],
  disabled: bool,
  loading: bool,
  label: string,
})
