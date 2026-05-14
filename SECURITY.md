# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in EveryMap-RS, please report it privately by emailing
**security@angoratek.com**. Do not open a public issue.

We aim to acknowledge reports within 48 hours and provide an initial assessment within 5 business days.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | Yes                |
| < 0.2.0 | No                 |

## Security Considerations

- **API keys**: Never commit API keys to version control. Set keys via environment variables
  (`EVERYMAP_HERE_API_KEY`, `EVERYMAP_GOOGLE_API_KEY`, `EVERYMAP_TOMTOM_API_KEY`,
  `EVERYMAP_MAPBOX_API_KEY`, `EVERYMAP_RADAR_API_KEY`) or the config file
  (`~/.everymap/config.toml`, with `[providers.<name>]` sections). The generic
  `EVERYMAP_API_KEY` env var serves as a fallback. API keys in memory are zeroized on drop
  via the `zeroize` crate.
- **TLS**: All provider APIs are accessed over HTTPS. The underlying HTTP client enforces TLS.
- **Unsafe code**: This project uses `#[deny(unsafe_code)]` at the workspace level. No `unsafe` blocks
  exist in the codebase.
