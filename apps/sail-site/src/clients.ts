// Clients seam for the SAIL docs site. No durable client state is needed — pages
// own their own scope, the shell owns the theme + base styles, and the
// component reference registers the primitive pack in its own route. This stays
// as the configured entry, intentionally empty.
export {}
