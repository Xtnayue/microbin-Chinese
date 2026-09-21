
![Screenshot](.github/index.png)

# MicroBin

![Build](https://github.com/szabodanika/microbin/actions/workflows/rust.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/microbin.svg)](https://crates.io/crates/microbin)
[![Docker Image](https://github.com/szabodanika/microbin/actions/workflows/release.yml/badge.svg)](https://hub.docker.com/r/danielszabo99/microbin)
[![Docker Pulls](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)

MicroBin is a super tiny, feature-rich, configurable, self-contained and self-hosted paste bin web application. It is very easy to set up and use, and will only require a few megabytes of memory and disk storage. It takes only a couple minutes to set it up, why not give it a try now?


### Host MicroBin yourself

Run our quick docker setup script ([DockerHub](https://hub.docker.com/r/danielszabo99/microbin)):
```bash
bash <(curl -s https://microbin.eu/docker.sh)
```

Or install it manually from [Cargo](https://crates.io/crates/microbin):

```bash
cargo install microbin;
curl -L -O https://raw.githubusercontent.com/szabodanika/microbin/master/.env;
source .env;
microbin
```

On our website [microbin.eu](https://microbin.eu), you will find the following:

- [Screenshots](https://microbin.eu/screenshots/)
- [Guide and Documentation](https://microbin.eu/docs/intro)
- [Roadmap](https://microbin.eu/roadmap)

## Features

- Entirely self-contained executable, MicroBin is a single file!
- Server-side encryption: with `read` set to password, the text content and all attachments are stored encrypted with that password
- File uploads (e.g. `server.com/file/pig-dog-cat`)
- Raw text serving (e.g. `server.com/raw/pig-dog-cat`)
- QR code support
- URL shortening and redirection
- Animal names instead of random numbers for upload identifiers (64 animals)
- Multiple attachments
- SQLite and JSON database support
- Three independent permissions (`read` / `write` / `list`) that can be combined freely
- Automatic dark mode and custom styling support with very little CSS and only vanilla JS (see [`water.css`](https://github.com/kognise/water.css))
- And much more!

## Changes from upstream

This fork is based on [`szabodanika/microbin`](https://github.com/szabodanika/microbin) v2.1.4. It adds a bilingual (English / Simplified Chinese) interface and reworks the permission model. The Chinese [`README.md`](README.md) documents this fork in more detail.

### Reworked permissions

The old fixed set of five privacy levels (`public` / `unlisted` / `readonly` / `private` / `secret`) is gone. Permissions are now three **independent** switches that can be granted separately:

| Permission | Values | Meaning | Applies to |
| --- | --- | --- | --- |
| `read` | `allow` / `password` | Who can see the content. With `password`, the text and all attachments are stored encrypted with that password on the server | `/upload/{id}`, `/raw/{id}`, `/file/{id}`, `/archive/{id}`, the edit page |
| `write` | `allow` / `password` / `refuse` | Who can modify or delete. `password` is an authorization check only and does **not** encrypt anything; `refuse` rejects everyone (the admin password can still delete) | saving edits, `/remove/{id}`, the "edit / delete" buttons |
| `list` | `yes` / `no` | Whether it appears on the public `/list` page | `/list` only |

Client-side (browser) encryption — the old `secret` level — has been **removed**; only server-side encryption remains.

Permission passwords are never stored. What is stored is a verification value `encrypt(upload id, password)`; a submitted password is accepted when decrypting that value yields the upload id. Two benefits:

1. It works regardless of whether the content is empty — this fixes the old bug where an attachment-only upload (empty text) could not have its password verified;
2. Verification cost is independent of content size.

The `read` and `write` passwords have separate jobs: the `read` password additionally encrypts/decrypts the content and attachments, while the `write` password only answers "is writing allowed?". They may be the same or different. See the `/guide` page (anchors `#read`, `#write`, `#list`, `#password`).

CLI flags changed accordingly:

- Removed: `--private`, `--enable-readonly`, `--default-privacy`, `--encryption-client-side`, `--encryption-server-side`
- Added: `--enable-unlisted`, `--enable-read-password`, `--enable-write-password`
- Added: `--default-permissions=read=allow,write=allow,list=yes` (comma-separated `key=value`)

**Upgrading is automatic.** Existing data is migrated on read by both backends, and the migration is idempotent (restarting does not migrate twice or alter already-migrated data):

- **SQLite** (default backend): `ALTER TABLE ... ADD COLUMN` plus a mapping pass at startup;
- **JSON** (`--json-db`): each record is recognized and completed before deserialization.

The mapping is (`read_only || encrypt_server` — **not** `read_only` alone — decides whether writing requires a password):

| Old field | New permission |
| --- | --- |
| `private` | `list = no` (else `yes`) |
| `encrypt_server` | `read = password` (else `allow`) |
| `read_only` \|\| `encrypt_server` | `write = password` (else `allow`) |
| `encrypted_key` | both `read_key` and `write_key` (same format, so old passwords keep working) |

The old model had no `refuse`, so migration never produces `write = refuse`. Both backends log a warning when a migration happens.

Note: content uploaded with the old browser-side encryption (the `secret` level) can no longer be decrypted in the browser and will remain ciphertext.

### Other upstream bugs fixed along the way

1. **The JSON backend ignored `--data-dir`** — the database path was hardcoded to `pasta_data/database.json`, so `--data-dir` had no effect with `--json-db`, and startup panicked whenever `pasta_data/` did not exist. It now follows `--data-dir`, like the SQLite backend.
2. **Editing a password-protected upload wrote the plaintext back to the database** — upstream's `post_edit_private` overwrote the stored value with the decrypted text, so the upload silently lost its password protection after the first edit. The content is now re-encrypted with the `read` password before being saved.
3. **Downloading a protected upload as a zip produced undecryptable ciphertext** — `/archive/{id}` used to stuff the `.enc` files straight into the zip; with browser-side decryption gone those files are useless. It now verifies the `read` password and returns the **decrypted** contents.

### Bilingual interface

A `--language` flag and a translation table in [`src/i18n.rs`](src/i18n.rs) add runtime `en` / `zh` switching for templates, server-side messages and CLI help.

## What is an upload?

In MicroBin, an upload can be:

- A text that you want to paste from one machine to another, e.g. some code,
- Files that you want to share, e.g. a video that is too large for Discord, a zip with a code project in it or an image,
- A URL redirection.

## When is MicroBin useful?

You can use MicroBin:

- To send long texts to other people,
- To send large files to other people,
- To share secrets or sensitive documents securely,
- As a URL shortener/redirect service,
- To serve content on the web, eg . configuration files for testing, images, or any other file content using the Raw functionality,
- To move files between your desktop and a server you access from the console,
- As a "postbox" service where people can upload their files or texts, but they cannot see or remove what others sent you,
- Or even to take quick notes.

...and many other things, why not get creative?

MicroBin and MicroBin.eu are available under the [BSD 3-Clause License](LICENSE).
