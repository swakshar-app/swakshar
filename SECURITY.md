# Security Policy

Swakshar decides when a legally valid digital signature is made, so security
reports get priority over everything else.

## Reporting a vulnerability

Please report privately through GitHub's **Report a vulnerability** button on
the repository's Security tab. Do not open a public issue.

Include what you found, how to reproduce it, and the impact you expect. We aim
to acknowledge within three working days and to agree a disclosure date with
you once a fix is ready.

## Scope

In scope: anything that could let a party other than the user obtain a
signature, learn the PIN, read PII from Swakshar's files or logs, make the
local certificate vouch for another site, or tamper with releases.

Out of scope: vulnerabilities in token vendors' drivers, the GST portal, or
browsers, unless Swakshar makes them worse.

## Supported versions

Until 1.0, only the latest release receives fixes.
