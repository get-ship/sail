# contrib — not the product

`helix-setup.sh` overlays **this checkout** onto one Helix install
(`~/.config/helix`, `target/debug/sail`, `cc`). It exists so maintainers
can dogfood. It is not how users install Sail, and it must not grow
into a product verb.

User-facing install lives in [../README.md](../README.md).
