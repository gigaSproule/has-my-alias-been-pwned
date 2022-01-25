# Has my alias been pwned?
[![CircleCI](https://circleci.com/gh/gigaSproule/has-my-alias-been-pwned/tree/main.svg?style=svg)](https://circleci.com/gh/gigaSproule/has-my-alias-been-pwned/tree/main)

A simple application to run through all email aliases for the given email alias service and check against [have i been pwned](https://haveibeenpwned.com/).

If the email alias has been leaked, then it is deactivated and printed out to console, so you can correct it.
## Usage
To be able to use the Have I Been Pwned API, an API token is needed. This can be provided by setting the `HIBP_TOKEN` environment variable (`.env` file is supported).

To use AnonAddy (the only supported email alias at the moment), the `ANONADDY_TOKEN` environment variable needs to be set (again, `.env` file is supported).

Simply run:
```bash
./has-my-alias-been-pwned
```
