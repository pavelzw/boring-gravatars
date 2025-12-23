# boring-gravatars

A Gravatar-compatible avatar service with boring-avatars support.

## Usage

Start the server:

```bash
cargo run
```

### Command Line Options

```
Options:
  -H, --host <HOST>          Host address to bind to [default: 0.0.0.0]
  -p, --port <PORT>          Port to listen on [default: 8000]
  -m, --max-size <MAX_SIZE>  Maximum allowed avatar size in pixels [default: 512]
  -h, --help                 Print help
  -V, --version              Print version
```

Example with custom settings:

```bash
cargo run -- --host 127.0.0.1 --port 3000 --max-size 1024
```

Request an avatar:

```
GET /avatar/{hash}?d={style}&s={size}
```

### Parameters

- `hash` - MD5 hash of the user's email (standard Gravatar format)
- `d` - Default/fallback style (optional, defaults to `identicon`)
- `s` - Size in pixels (optional, defaults to 80, max configurable via `--max-size`)

### Supported Styles

**Gravatar styles** (proxied directly to Gravatar):
- `404` - Return 404 if no Gravatar exists
- `mp` - Mystery person silhouette
- `identicon` - Geometric pattern
- `monsterid` - Generated monster
- `wavatar` - Generated face
- `retro` - 8-bit style
- `robohash` - Robot
- `blank` - Transparent image

**Boring Avatars styles**:
- `marble` - Marble pattern
- `beam` - Simple face
- `pixel` - Pixelated
- `sunset` - Gradient circles
- `ring` - Concentric rings
- `bauhaus` - Geometric shapes

### Example

```bash
# Get avatar with marble fallback, 128px
curl "http://localhost:8000/avatar/abc123?d=marble&s=128" > avatar.png
```

## Building

```bash
cargo build --release
```
