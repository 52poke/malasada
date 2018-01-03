malasada
[![Malasada](https://cdn.bulbagarden.net/upload/8/8e/Bag_Big_Malasada_Sprite.png)](https://bulbapedia.bulbagarden.net/wiki/Malasada)
=========

[![NPM version](https://img.shields.io/npm/v/malasada.svg)](https://npmjs.org/package/malasada)
[![build status](https://img.shields.io/travis/mudkipme/malasada.svg)](https://travis-ci.org/mudkipme/malasada)
[![node version](https://img.shields.io/badge/node.js-%3E=_8.5-green.svg)](https://nodejs.org/en/download/)

A reverse proxy server to convert, cache and respond images in WebP format. Can be used standalone or as a [Koa](http://koajs.com) middleware.

## Install

[![NPM](https://nodei.co/npm/malasada.png?downloads=true)](https://nodei.co/npm/malasada/)

You'll need to install WebP binaries on your server, please see [node-webp](https://www.npmjs.com/package/cwebp) manual for installation.

### Docker

You can also run malasada with Docker:

```bash
docker run --name malasada -p 80:3002 -v /path/to/config.json:/app/config.json -v /path/to/cache:/app/cache -d mudkip/malasada
```

Replace `/path/to` with the correct location of `config.json` and cache.

## Configruation

A `config.json` file is required to run this program.

```json
{
    "backendType": "remote",
    "backend": "http://upload.wikimedia.org",
    "maxAge": 3628800000,
    "forceWebP": false,
    "cacheRootPath": "/var/cache/malasada",
    "requestHeaders": {},
    "queue": 1,
    "port": 3002
  }
```

* **backendType**: Can be `remote` or `local`.
* **backend**: May be a http or https host when `backendType` is `remote`, or a local directory when `backendType` is `local`. **Required.**
* **maxAge**: The max age (in second) sent to browsers.
* **forceWebP**: When is true, the server always response WebP images when possible, otherwise it only respond WebP images when it is accepted by browsers in `Accept` header.
* **cacheRootPath**: Where the cache files saves, defaults to `cache` directory of this program.
* **requestHeaders**: Additional headers sent to backend, only useful when `backendType` is `remote`.
* **queue**: Simultaneous WebP conversion tasks, defaults to `1`.
* **port**: The listen port of the server, only useful when used standalone.

## Usage

When installed globally, it can be run with `malasada --config config.json`.

You can also use it as a middleware in your Koa instance:
```js
const Koa = require('koa');
const malasada = require('malasada');

const app = new Koa();
app.use(malasada(config));
```

Remember this program is not designed to be a full-featured reverse proxy, the backend server should only be a static file server.

### Cache Deletion

A `PURGE` request, or a `GET` request with `/purge` prefix can be used to delete cache of a certain path.

## License

[MIT](LICENSE)

## Notice

While the sweet [malasada](https://wiki.52poke.com/wiki/%E9%A6%AC%E6%8B%89%E8%96%A9%E9%81%94%E9%80%A3%E9%8E%96%E5%BA%97) is used on [52Poké Wiki](https://wiki.52poke.com/) to handle hundreds of thousands requests daily. It currently lacks tests and significant features such as proper cache control and cache size management.

Pull requests and issues are welcome.