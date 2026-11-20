# Security policy

A browser handles people's logins, payments and private data, so we take security reports seriously.

## Reporting a vulnerability
**Please don't open a public issue.** Report it privately through GitHub instead: on the repository's **Security** tab, click **Report a vulnerability**. Only the maintainer can see the report.

Please include:
- what's affected (browser version, OS) and what an attacker could do
- steps or a minimal page to reproduce it
- whether it needs the user to do something (open a page, click, install a file)

You'll get an acknowledgement within a few days. Please give us reasonable time to fix the issue before you disclose it publicly. We're happy to credit you in the release notes.

## Supported versions
The project is in early development and has no releases yet. Once releases exist, only the latest release gets security fixes.

## Scope
In scope: this repository's code, including the browser shell, internal `lwb://` pages, the IPC between them, and the ad blocker.

Out of scope: bugs in the operating system's web engine itself (WebView2, WKWebView, WebKitGTK). Report those to Microsoft, Apple or the WebKitGTK project. Do tell us if our code makes such a bug easier to exploit.
