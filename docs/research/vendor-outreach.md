# Draft vendor outreach

Status: draft, 2026-10-07. The owner sends these, edits freely, and fills the
bracketed blanks. Goal: let Swakshar work with the token without the user
installing a separate driver, by one of: permission to redistribute the macOS
PKCS#11 module inside the app, a published card command spec so an open driver
can be written, or an OpenSC driver contribution (as Feitian did for
ePass2003). Swakshar is open source (Apache-2.0) and signs only with the
user's own token after explicit approval; it is not affiliated with any
Certifying Authority.

## Hypersecu (HYP2003)

To: support@hypersecu.com

Subject: Redistributing the HYP2003 macOS PKCS#11 module in an open-source signer

Hello,

I maintain Swakshar, an open-source macOS signer for India's GST portal that
uses the user's own DSC token through its PKCS#11 driver. It is Apache-2.0 and
signs only after the user approves each request.

I would like HYP2003 owners to use the app without a separate driver install.
Three options would each achieve that, any one is fine:

1. Permission to bundle your macOS PKCS#11 module (`libcastle`) inside our
   signed app, with your copyright and licence shown. Is there a redistribution
   licence for this?
2. A published command specification for the token, so an open driver can be
   written without touching your binary.
3. Interest in an OpenSC driver for the HYP2003, as Feitian did for ePass2003.

Could you also confirm the current macOS module's filename and path, and
whether it ships universal (arm64 and x86_64)?

Thank you,
[name], Swakshar, [url]

## Watchdata (ProxKey)

To: [Watchdata India contact, via the issuing CA if no direct email]

Subject: ProxKey macOS PKCS#11 driver for an open-source signer

Hello,

I maintain Swakshar, an open-source Apache-2.0 macOS signer for India's GST
portal. It signs with the user's own DSC token through PKCS#11, only after the
user approves each request.

ProxKey users currently install the Watchdata macOS driver by hand. To make
that smoother I am looking for one of:

1. A redistribution licence to include your macOS PKCS#11 module in our signed
   app, with your copyright and licence shown.
2. A published command specification so an open driver can be written.
3. The current macOS driver's download URL, exact PKCS#11 filename and path,
   and whether it is universal (arm64 and x86_64), if redistribution is not
   possible, so the app can at least guide users to the right file.

Thank you,
[name], Swakshar, [url]

## Longmai / Century (mToken CryptoID)

To: [Longmai support, via longmai.net]

Subject: mToken CryptoID on macOS, PKCS#11 and OpenSC

Hello,

I maintain Swakshar, an open-source Apache-2.0 macOS signer that uses the
user's own DSC token through PKCS#11, only after the user approves each
request.

Your brochure describes the mToken CryptoID as a CCID plug-and-play device. To
support it without a manual driver install, could you tell me:

1. On macOS, is the token usable through the system CCID driver alone, with no
   vendor kernel or user driver?
2. Is the on-token layout standard PKCS#15, so OpenSC's generic path can read
   it? If you maintain or would accept an OpenSC driver, I would like to help.
3. If a vendor PKCS#11 module is required on macOS, is there a redistribution
   licence to bundle it in our signed app, and what is its filename and path?

Thank you,
[name], Swakshar, [url]

## TrustKey

To: [issuing Certifying Authority or reseller]

Subject: TrustKey macOS driver and PKCS#11 module name

Hello,

I maintain Swakshar, an open-source Apache-2.0 macOS signer for India's GST
portal, using the user's own DSC token through PKCS#11 after explicit
approval.

For TrustKey tokens I am trying to confirm the basics so users do not have to
guess:

1. Who manufactures the TrustKey token, and is there a macOS PKCS#11 driver?
2. The exact PKCS#11 module filename and install path on macOS, and whether it
   is universal (arm64 and x86_64).
3. Whether the maker allows redistribution of the macOS module in a signed
   open-source app, or publishes a card command specification.

Thank you,
[name], Swakshar, [url]
