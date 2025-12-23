# boring-gravatars

A Gravatar-compatible avatar service with boring-avatars support.

## Usage

Start the server:

```bash
cargo run
```

The server listens on `http://0.0.0.0:8000`.

Request an avatar:

```
GET /avatar/{hash}?d={style}&s={size}
```

### Parameters

- `hash` - MD5 hash of the user's email (standard Gravatar format)
- `d` - Default/fallback style (optional, defaults to `identicon`)
- `s` - Size in pixels (optional, defaults to 80, max 512)

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
