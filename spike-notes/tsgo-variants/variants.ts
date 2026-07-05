// A real, typed variants() — the compiler would generate the <ButtonProps>
// param from the component's <prop values="…"> declarations.
export type SxString = string

export type VariantsConfig<V extends Record<string, string>> = {
  variants?: { [P in keyof V]?: { [Value in V[P]]?: SxString } }
  compound?: Array<{ when: Partial<V>; sx: SxString }>
}

export function variants<V extends Record<string, string>>(
  part: string,
  config: VariantsConfig<V>,
): VariantsConfig<V> {
  return config
}
