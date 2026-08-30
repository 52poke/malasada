malasada
[![Malasada](https://archives.bulbagarden.net/media/upload/8/8e/Bag_Big_Malasada_Sprite.png)](https://bulbapedia.bulbagarden.net/wiki/Malasada)
=========

A high-performance image processing server for [52Poké Wiki](https://wiki.52poke.com/), written in Rust.

It replaces the previous Serverless/Lambda implementation and provides two main features:

- **Dynamic Thumbnail Generation:** Generates thumbnails on-the-fly for MediaWiki images stored in S3.
- **WebP Conversion:** Converts images to WebP format to reduce bandwidth costs, serving them from a separate cache path.

## Pre-requisites

This service is designed for [MediaWiki](https://www.mediawiki.org/) installations that store images in an S3-compatible object storage (e.g., AWS S3, Linode Object Storage).

## Configuration

The application is configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `S3_BUCKET` | The name of the S3 bucket storing images. | `media.52poke.com` |
| `S3_REGION` | The S3 region. | `ap-northeast-1` |
| `S3_ENDPOINT` | Custom S3 endpoint URL (optional). | `https://jp-osa-1.linodeobjects.com` |
| `S3_ACCESS_KEY_ID` | AWS Access Key ID. | (Required if not using instance roles) |
| `S3_SECRET_ACCESS_KEY` | AWS Secret Access Key. | (Required if not using instance roles) |
| `PORT` | The port the server listens on. | `3000` |

## Running the Server

### Docker (Recommended)

Images are available on GitHub Container Registry.

```bash
docker run -d \
  -p 3000:3000 \
  -e S3_BUCKET=my-wiki-images \
  -e S3_REGION=us-east-1 \
  -e S3_ACCESS_KEY_ID=... \
  -e S3_SECRET_ACCESS_KEY=... \
  ghcr.io/mudkipme/malasada:latest
```

### Local Development

Prerequisites: Rust 1.94.1+

```bash
# Install dependencies and run
cargo run --release
```

## Deployment Integration

### MediaWiki Configuration

This function assumes images are stored in a `wiki/` path within a single S3 bucket, with `hashLevels` set to `2`.

```php
$wgFileBackends["s3"] = [
    'class'              => 'AmazonS3FileBackend',
    'name'               => 'AmazonS3',
    'wikiId'             => 'wiki',
    // ... other config ...
    'containerPaths'     => [
        'wiki-local-public'  => '<s3-bucket>/wiki',
        'wiki-local-thumb'   => '<s3-bucket>/wiki/thumb',
        // ...
    ]
];

$wgLocalFileRepo  =  [
    'class'              => 'LocalRepo',
    'backend'            => 'AmazonS3',
    'transformVia404'    => true, // Important: Let Malasada handle missing thumbnails
    // ...
];
```

### Nginx Configuration

Configure Nginx to proxy 404 errors from your static asset domain to the Malasada service.

#### Standard Thumbnails

```nginx
location / {
    # Serve directly from S3 first
    proxy_pass http://<s3-bucket>.<s3-region>.amazonaws.com;
    proxy_redirect off;
    proxy_intercept_errors on;
    # On 404 (missing file), try generating it via Malasada
    error_page 404 = @malasada;
}

location @malasada {
    internal;
    # Proxy to the running Malasada instance
    proxy_pass http://localhost:3000$request_uri; 
}
```

#### WebP Delivery

```nginx
location / {
    # Check WebP cache in S3 first
    proxy_pass http://<s3-bucket>.<s3-region>.amazonaws.com/webp-cache$request_uri;
    proxy_redirect off;
    proxy_intercept_errors on;
    error_page 404 = @malasada_webp;
}

location @malasada_webp {
    internal;
    # Proxy to Malasada WebP endpoint
    proxy_pass http://localhost:3000/webp$request_uri;
}
```

### Cache Purging

To purge the WebP cache when an image is updated or deleted, send a `DELETE` request to the `/webp/...` endpoint.

```bash
curl -X DELETE http://localhost:3000/webp/wiki/path/to/image.png
```

## License

[MIT](LICENSE)
