# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security issue, please **do not** open a public GitHub issue.

Instead, report it privately via [GitHub Security Advisories](https://github.com/AliSawari/aether/security/advisories/new) or by contacting the maintainer through their GitHub profile.

Please include:

- A description of the issue and potential impact
- Steps to reproduce
- Affected version(s)

You should receive a response within a reasonable timeframe. Critical issues will be prioritized for patch releases.

## Scope Notes

Aether runs privileged WireGuard operations via `sudo`/`pkexec` and a bundled `wgctl.sh` script. Review elevation rules (`polkit/`, sudoers installed by **Authorize once**) before deploying in sensitive environments.
