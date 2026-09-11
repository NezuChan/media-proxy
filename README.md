<div align="center">

<img src="https://i.kagchi.my.id/nezuko.png" alt="Logo" width="200px" height="200px" style="border-radius:50%"/>

# @nezuchan/media-proxy

**A fast media proxy service that resizes images and streams audio, written in Rust.**

[![GitHub](https://img.shields.io/github/license/nezuchan/media-proxy)](https://github.com/nezuchan/media-proxy/blob/main/LICENSE)
[![Discord](https://discordapp.com/api/guilds/785715968608567297/embed.png)](https://nezu.my.id)

</div>

## Overview

`media-proxy` hides origin URLs behind an `aes-256-cbc` encrypted payload. Clients
pass a hex-encoded, encrypted URL and the service fetches, transforms and returns
the media. It can proxy **images** (resize + crop) and **audio** (byte-range
passthrough).

- Pure Rust image pipeline (no libvips / native image dependencies)
- Bounded in-memory cache for processed images
- Docker ready
- Production Ready

## Requirements

- Rust 1.75+ (stable)
- No system libraries required — TLS uses `rustls`

## Usage

```bash
cp .env_example .env
cargo run --release
```

## Routes

| Method | Path | Description |
| ------ | ---- | ----------- |
| `GET` | `/health` | Liveness probe, returns `{"status":"ok"}` |
| `GET` | `/image/:size/:media` | Resize/crop an image. `size` is `WxH`, `media` is hex(AES-CBC(url)). Returns `image/jpeg`. |
| `GET` | `/audio/:media` | Stream audio. `media` is hex(AES-CBC(url)). Forwards `Range`/`206` and the upstream content type. |

### Errors

Errors use the original JSON shape:

```json
{ "statusCode": 400, "message": "invalid width size" }
```

## Configuration

| Variable | Default | Description |
| -------- | ------- | ----------- |
| `KEY` | — | AES-256 key, exactly 32 bytes |
| `IV` | — | AES-CBC IV, exactly 16 bytes |
| `MAX_WIDTH` | `1024` | Maximum requested width |
| `MAX_HEIGHT` | `1024` | Maximum requested height |
| `IMAGE_QUALITY` | `100` | JPEG re-encode quality (1-100) |
| `COMPRESS_IMAGE` | `false` | Route image origins through `wsrv.nl` |
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `3000` | Bind port |
| `CACHE_MAX_BYTES` | `67108864` | Image cache capacity (approx. bytes) |
| `CACHE_TTL_SECS` | `300` | Image cache time-to-live |
| `RUST_LOG` | `info` | Log filter directive |

## Image behaviour

- Decodes the origin bytes with the pure-Rust `image` crate.
- Square images are scaled with a nearest-neighbour kernel.
- Non-square images are coerced to a square box and centre-cropped
  (approximating libvips `InterestingCentre`).
- Output is always JPEG at `IMAGE_QUALITY`.

## Docker

```bash
docker build -t media-proxy .
docker run --env-file .env -p 3000:3000 media-proxy
```
