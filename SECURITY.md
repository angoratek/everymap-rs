# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in EveryMap-RS, please report it privately by emailing
**security@angoratek.com**. Do not open a public issue.

We aim to acknowledge reports within 48 hours and provide an initial assessment within 5 business days.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | Yes                |
| < 0.4.0 | No                 |

## Security Considerations

- **API keys**: Never commit API keys to version control. The benchmark reads the
  per-provider variables (`EVERYMAP_HERE_API_KEY`, `EVERYMAP_GOOGLE_API_KEY`,
  `EVERYMAP_TOMTOM_API_KEY`, `EVERYMAP_MAPBOX_API_KEY`, `EVERYMAP_MAPQUEST_API_KEY`,
  `EVERYMAP_RADAR_API_KEY`) and the keyless deployment variables
  (`EVERYMAP_OSRM_BASE_URL`, `EVERYMAP_VALHALLA_BASE_URL`); the CLI reads
  `EVERYMAP_API_KEY` and the config file (`~/.everymap/config.toml`, with
  `[providers.<name>]` sections). API keys in memory are zeroized on drop via the
  `zeroize` crate.
- **TLS**: Hosted provider APIs are accessed over HTTPS. The self-hosted OSRM and
  Valhalla defaults may be plain HTTP on a local deployment.
- **Unsafe code**: This project uses `#[deny(unsafe_code)]` at the workspace level. No `unsafe` blocks
  exist in the codebase.
- **Removed capture artefact (November 2026)**: a recorded MapQuest directions
  response carried the session id from the capture. The repository history and the
  release tags were rewritten to replace it, and the hygiene gate now refuses a
  session or signature token. Versions 0.4.1, 0.4.2 and 0.4.3 are published and
  carry the corrected payload. The 0.3.0–0.4.0 `everymap-providers-mapquest`
  tarballs on crates.io predate the rewrite and cannot be changed; they are
  superseded by 0.4.1 and later. The owner approved yanking them on 2026-11-10;
  the yank is pending a registry token on the machine that runs it, because the
  token is held only by the release workflow. The release workflow publishes
  crates and attaches binaries, and does not yank.
