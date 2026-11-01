
![Screenshot](.github/index.png)

# MicroBin

![Build](https://github.com/szabodanika/microbin/actions/workflows/rust.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/microbin.svg)](https://crates.io/crates/microbin)
[![Docker Image](https://github.com/szabodanika/microbin/actions/workflows/release.yml/badge.svg)](https://hub.docker.com/r/danielszabo99/microbin)
[![Docker Pulls](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)

MicroBin is a super tiny, feature-rich, configurable, self-contained and self-hosted paste bin web application. It is very easy to set up and use, and will only require a few megabytes of memory and disk storage. It takes only a couple minutes to set it up, why not give it a try now?


### Host MicroBin yourself

Run the docker setup script from this repository (it also downloads the matching
`.env` and `compose.yaml`):

```bash
bash <(curl -s https://raw.githubusercontent.com/Xtnayue/microbin-Chinese/master/docker-setup.sh)
```

Or run the image directly:

```bash
docker run -d --name microbin -p 8080:8080 \
  -v microbin_data:/app/microbin_data \
  ghcr.io/xtnayue/microbin-chinese:latest
```

The Chinese README ([README.md](README.md)) documents this fork's differences:
per-request language negotiation and the three independent permissions
(`read` / `write` / `list`).

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
| `read` | `allow` / `password` | Who can see the content. With `password`, the text and all attachments are stored encrypted with that password on the server | `/upload/{id}`, `/raw/{id}`, `/file/{id}`, `/archive/{id}`, the URL short link `/url/{id}` and `/u/{id}`, the edit page |
| `write` | `allow` / `password` / `refuse` | Who can modify or delete. `password` is an authorization check only and does **not** encrypt anything; `refuse` rejects everyone (the admin password can still delete) | saving edits, `/remove/{id}`, the "edit / delete" buttons |
| `list` | `yes` / `no` | Whether it appears on the public `/list` page | `/list` only |

Client-side (browser) encryption — the old `secret` level — has been **removed**; only server-side encryption remains.

The shipped configuration templates were brought in line as well: the five variables that went away with the old model (`MICROBIN_PRIVATE`, `MICROBIN_ENABLE_READONLY`, `MICROBIN_ENCRYPTION_CLIENT_SIDE`, `MICROBIN_ENCRYPTION_SERVER_SIDE`, `MICROBIN_DEFAULT_PRIVACY`) are gone from both `.env` and `compose.yaml`, which now carry `MICROBIN_ENABLE_UNLISTED`, `MICROBIN_ENABLE_READ_PASSWORD`, `MICROBIN_ENABLE_WRITE_PASSWORD`, `MICROBIN_DEFAULT_PERMISSIONS` and `MICROBIN_LANGUAGE` instead. These two files are exactly what `docker-setup.sh` downloads and feeds to `docker compose`, so leaving them stale means an operator sets a variable that does nothing — silently, with no error anywhere.

Permission passwords are never stored. What is stored is a verification value `encrypt(upload id, password)`; a submitted password is accepted when decrypting that value yields the upload id. Two benefits:

1. It works regardless of whether the content is empty — this fixes the old bug where an attachment-only upload (empty text) could not have its password verified;
2. Verification cost is independent of content size.

The `read` and `write` passwords have separate jobs: the `read` password additionally encrypts/decrypts the content and attachments, while the `write` password only answers "is writing allowed?". They may be the same or different. See the `/guide` page (anchors `#read`, `#write`, `#list`, `#password`).

CLI flags changed accordingly:

- Removed: `--private`, `--enable-readonly`, `--default-privacy`, `--encryption-client-side`, `--encryption-server-side`
- Added: `--enable-unlisted`, `--enable-read-password`, `--enable-write-password`
- Added: `--default-permissions=read=allow,write=allow,list=yes` (comma-separated `key=value`)
- Activated: `--default-view=gallery|list|stream` (the flag existed upstream, but the template hardcoded the value, so it never had any effect — see "Other upstream bugs fixed along the way" below)

> The three `--enable-*` flags are **both a form switch and a server-side policy**. Turning one off removes the option from the new-upload form, and posting that disabled value straight to `/upload` is rejected with **HTTP 400** plus a readable message. All three default to `true`, in which case these guards never fire and behaviour is exactly as before.
>
> Why reject rather than do what upstream did — **silently downgrade**? A downgrade turns all three cases into a result that is **more open than the user asked for**: posting `read=password` and getting `allow` means the user believes the content is encrypted when it is stored in plaintext; posting `write=password` and getting `allow` means the user believes nobody else can modify it when anybody can; posting `list=no` and getting `list=yes` means the user believes the upload is not publicly listed when it is on `/list`. The severity differs, but the direction is the same: the protection the user asked for did not take effect. Upstream's `--enable-readonly` was exactly this pattern (posting `readonly` after disabling it produced a publicly writable upload). Rejection collapses all three into one outcome — no upload is created, the client gets a 400 and a readable message, and there is no state in which the user thinks something is protected when it is not.
>
> There is also a startup self-check: if `--default-permissions` selects a level that happens to be disabled (e.g. `read=password` with `--enable-read-password=false`), the form does not render that option and the browser silently falls back to the first option in the group. The program does **not** change behaviour — it only logs a `WARN` at startup naming the level that will actually apply.

**Upgrading is automatic.** Existing data is migrated on read by both backends, and the migration is idempotent (restarting does not migrate twice or alter already-migrated data):

- **SQLite** (default backend): `ALTER TABLE ... ADD COLUMN` plus a mapping pass at startup;
- **JSON** (`--json-db`): each record is recognized and completed before deserialization.

> ⚠️ The old JSON backend hardcoded its database path to `pasta_data/database.json`, **ignoring `--data-dir`**. This version makes it follow `--data-dir` and performs a **one-time relocation**: if the new location has no database file yet while `pasta_data/database.json` exists, it is moved over and a warning is logged — so existing `--json-db` users do **not** end up staring at an empty database. Conversely, when the new location already has a database file, the old file is left untouched, so nothing is ever overwritten.

The mapping is (`read_only || encrypt_server` — **not** `read_only` alone — decides whether writing requires a password):

| Old field | New permission |
| --- | --- |
| `private` | `list = no` (else `yes`) |
| `encrypt_server` | `read = password` (else `allow`) |
| `read_only` \|\| `encrypt_server` | `write = password` (else `allow`) |
| `encrypted_key` | both `read_key` and `write_key` (same format, so old passwords keep working) |

The old model had no `refuse`, so migration never produces `write = refuse`. Both backends log a warning when a migration happens.

Note: content uploaded with the old browser-side encryption (the `secret` level) can no longer be decrypted in the browser and will remain ciphertext.

### Responsive layout

Upstream pinned `<body>` to `max-width: 800px` (1080px with `--wide`), so a wide window only ever showed a narrow centred column — multi-column tables such as the admin page were squeezed into a tall strip.

The page now **fills the whole window by default** (`--wide` is on by default). Use `--max-width` to constrain it to a pixel value (it wins over `--wide`), or `--wide=false` to fall back to a 1600px centred column. The background is now painted on `<html>` as well, so short pages no longer show a second colour below the content.

Two layout bugs were fixed along the way: only the write and list permission labels carried a `margin-top`, which put them 9.6px lower than the read label; and the fixed row height of `#settings` let content overflow once the "requires a password" option revealed the extra password field.

### Other upstream bugs fixed along the way

Five upstream leftovers were found and fixed while reworking the permission model:

1. **The JSON backend ignored `--data-dir`** — the database path was hardcoded to `pasta_data/database.json`, so `--data-dir` had no effect with `--json-db`, and startup panicked whenever `pasta_data/` did not exist (the Docker image only creates `microbin_data/`, so `--json-db` in a container was effectively broken on startup). It now follows `--data-dir`, like the SQLite backend, and relocates the old file once (see above).
2. **Editing a password-protected upload wrote the plaintext back to the database** — upstream's `post_edit_private` overwrote the stored value with the decrypted text, so the upload silently lost its password protection after the first edit. The content is now re-encrypted with the `read` password before being saved.
3. **Downloading a protected upload as a zip produced undecryptable ciphertext** — `/archive/{id}` used to stuff the `.enc` files straight into the zip; with browser-side decryption gone those files are useless. It now verifies the `read` password and returns the **decrypted** contents.
4. **URL short links ignored the read permission and silently broke** — `/url/{id}` and `/u/{id}` returned `content` verbatim as the `Location` header. With `read=password` that `content` is ciphertext, so the browser was redirected onto a blob of ciphertext and the link was dead (not a plaintext leak). They now redirect to the password page `/auth_url/{id}` first, and only decrypt and forward to the target once the password checks out. `read=allow` short links behave exactly as before.
5. **`--default-view` was a dead flag** — the flag existed upstream (defined in `args.rs`, visible in `--help`), but `templates/upload.html` hardcoded its fallback to `'gallery'` and nothing in the repository ever read it, so setting it did nothing. The value is now rendered into the page script by the server and **whitelisted** (only `stream` / `list` / `gallery` are accepted, anything else falls back to `gallery`). The whitelist is not optional: the value is interpolated directly into a JS string literal (the template declares `escape = "none"`), so passing it through verbatim would let `--default-view "x'; alert(1)//"` break the entire page script. An explicit `?view=` in the URL still takes precedence over this default.

### Bilingual interface

A `--language` flag and a translation table in [`src/i18n.rs`](src/i18n.rs) provide `en` / `zh` support for templates and server-side messages. The language is **negotiated per request**: `--language` is only the fallback, and each visitor can override it with `?lang=`, the `microbin_lang` cookie, or the `Accept-Language` header (in that order of precedence). The `--help` output has no request context, so it is localised from `--language` alone.

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
