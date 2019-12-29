malasada
[![Malasada](https://cdn.bulbagarden.net/upload/8/8e/Bag_Big_Malasada_Sprite.png)](https://bulbapedia.bulbagarden.net/wiki/Malasada)
=========

A serverless function to resize and convert images on [52Poké Wiki](https://wiki.52poke.com/).

It is inspired by [this example](https://github.com/serverless/examples/tree/master/aws-node-dynamic-image-resizer) of Serverless Framework, and contains two features:

- Generate thumbnails for media files on MediaWiki
- Convert images to WebP to reduce bandwidth costs

On Wikimedia wikis, [Thumbor](https://wikitech.wikimedia.org/wiki/Thumbor) is used for similar tasks.

## Pre-requisites

This function is designed for use on [MediaWiki](https://www.mediawiki.org/) installations which have [AWS extension](https://www.mediawiki.org/wiki/Extension:AWS) configured to store images in S3. You also need the following:

* [AWS Credentials](https://serverless.com/framework/docs/providers/aws/guide/credentials/) for Serverless Framework.
* Docker and docker-compose installed locally.

## Deployment

### MediaWiki

This function assumes the images is stored in `wiki/` path in a single S3 bucket, `hashLevels` be `2`, and generating thumbnail on parse be disabled.

```php
$wgFileBackends['s3'] = [
    'class'              => 'AmazonS3FileBackend',
    'name'               => 'AmazonS3',
    'wikiId'             => 'wiki',
    'lockManager'        => 'redisLockManager',  // remove this if a lock manager isn't in use
    'containerPaths'     => [
        'wiki-local-public'  => '<s3-bucket>/wiki',
        'wiki-local-thumb'   => '<s3-bucket>/wiki/thumb',
        'wiki-local-temp'    => '<s3-bucket>/wiki/temp',
        'wiki-local-deleted' => '<s3-bucket>/wiki/deleted',
    ]
];

$wgLocalFileRepo  =  [
    'class'              => 'LocalRepo',
    'name'               => 'local',
    'backend'            => 'AmazonS3',
    'scriptDirUrl'       => $wgScriptPath,
    'url'                => wfScript( 'img_auth' ),
    'hashLevels'         => 2,
    'deletedHashLevels'  => 3,
    'zones'             =>  [
        'public'  => [ 'url' => 'https://<s3-public-url>/wiki' ],
        'thumb'   => [ 'url' => 'https://<s3-public-url>/wiki/thumb' ],
        'temp'    => [ 'url' => false ],
        'deleted' => [ 'url' => false ]
    ],
    'transformVia404' => true
];

$wgGenerateThumbnailOnParse = false;

wfLoadExtension( 'AWS' );
$wgAWSCredentials = [
        'key'    => '<aws-api-key>',
        'secret' => '<aws-secret-key>',
        'token'  => false
];
$wgAWSRegion = '<s3-region>';
```

On 52Poké Wiki, we serve original images for logged-in users, and WebP-compressed images for anonymous users. This is configured via [Lazyload](https://github.com/mudkipme/mediawiki-lazyload#configuration) extension.

### Malasada

1. Clone this repository and add AWS credentials and S3 configuration into `secrets/secrets.env` file.

2. Deploy malasada:

```
docker-compose up --build
```

### Nginx

Nginx (or any other front-end web server) needs to be configured to proxy requests to S3 and handle 404 errors to this lambda function.

This example uses separated domain for original and WebP images, but `Accept` header may also be considered to use.

#### For domains without WebP compression:

```nginx
location / {
    add_header Access-Control-Allow-Methods GET;
    proxy_pass http://<s3-bucket>.<s3-region>.amazonaws.com;
    proxy_redirect     off;
}

location ~ ^/wiki/thumb/(archive/)?[0-9a-f]/[0-9a-f][0-9a-f]/([^/]+)/([0-9]+)px-.*$ {
    proxy_pass http://<s3-bucket>.<s3-region>.amazonaws.com;
    proxy_redirect     off;
    proxy_intercept_errors on;
    error_page     404 502 504 = @thumb;
}

location @thumb {
    internal;
    proxy_pass         https://<api-gateway-endpoint>/<api-gateway-stage>$request_uri;
    proxy_ssl_server_name on;
}

location ~ ^/wiki/deleted/ {
    deny all;
}
```

#### For domains with WebP compression:

```nginx
location / {
    proxy_pass http://<s3-bucket>.<s3-region>.amazonaws.com/webp-cache$request_uri;
    proxy_redirect     off;
    proxy_intercept_errors on;
    error_page     404 502 504 = @webp;
    break;
}

location @webp {
    internal;
    proxy_pass https://<api-gateway-endpoint>/<api-gateway-stage>/webp$request_uri;
    proxy_ssl_server_name on;
}

location ~ ^/wiki/deleted/ {
    deny all;
}
```

### Cache Purging

WebP cache needs be deleted when the upstream media file is updated. A `DELETE` request can be sent to `https://<api-gateway-endpoint>/<api-gateway-stage>/webp/<s3 object path>` to delete the WebP cache.

On 52Poké Wiki, we use [timburr](https://github.com/mudkipme/timburr) to handle cache purging.

```yaml
purge:
  entries:
  - host: <s3-public-url>
    method: DELETE
    uris:
    - "https://<api-gateway-endpoint>/<api-gateway-stage>/webp#url#"

rules:
- name: purge
  topic: cdn-url-purges
  taskType: purge
```

## Configuration

Bucket and region settings can be configured via `secrets/secrets.env`. Feel free to look into and modify the TypeScript files and `serverless.yml` to customize this project for your own needs.

## Limitation

Currently malasada don't prevent simultaneous requests for generating same object. A locking mechanism (either based on SQS or DynamoDB) may be considered to implement.

## License

[MIT](LICENSE)
