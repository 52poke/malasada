import path from 'path';
import axios from 'axios';
import _ from 'lodash';
import send from 'koa-send';
import fs from 'mz/fs';
import normalizeHeaderCase from 'header-case-normalizer';
import mime from 'mime-types';
import Cache from './cache';
import defaultPath from './defaultPath';
import Queue from './queue';

export default function(options) {
  options = Object.assign({}, {
    backendType: 'remote',
    backend: 'http://upload.wikimedia.org',
    maxAge: 3628800 * 1000,
    forceWebP: false,
    cacheRootPath: defaultPath.cacheRootPath,
    requestHeaders: {},
    queue: 1
  }, options);

  const caching = new Set();
  const queue = new Queue(options);

  return async function (ctx, next) {

    // Support purge
    const purge = async (path) => {
      const cache = new Cache(path, options);
      const result = await cache.purge();
      if (!result) {
        ctx.throw(404, 'File not found.');
      }
      ctx.body = null;
    };

    if (ctx.path.indexOf('/purge/') === 0) {
      await purge(ctx.path.replace(/^\/purge/, ''));
      return;
    } else if (ctx.method === 'PURGE') {
      await purge(ctx.path);
      return;
    }

    const mimeType = mime.lookup(ctx.path);
    const responseWebP = mimeType && mimeType.match(/^image\//)
      && (options.forceWebP || (ctx.get('accept') && ctx.get('accept').indexOf('webp') !== -1));

    // Send the cached WebP file if exists
    const cache = new Cache(ctx.path, options);
    if (responseWebP) {
      const dest = await cache.dest();
      if (await fs.exists(dest)) {
        await send(ctx, path.relative(options.cacheRootPath, dest), {
          root: options.cacheRootPath,
          maxAge: options.maxAge
        });
        return;
      }
    }

    const source = await cache.source();

    if (options.backendType === 'local') {
      if (!await fs.exists(source)) {
        return await next();
      }
      await send(ctx, ctx.path, { root: options.backend, maxAge: options.maxAge });
      if (responseWebP && !queue.has(ctx.path)) {
        queue.add(ctx.path);
      }
      return;
    }

    // Send the original file and convert to WebP
    let response = null;
    try {
      response = await axios.get(ctx.path, {
        baseURL: options.backend,
        headers: options.requestHeaders,
        responseType: 'stream'
      });
      _.each(response.headers, (value, key) => {
        ctx.set(normalizeHeaderCase(key), value);
      });
      ctx.body = response.data;
    } catch (error) {
      if (!error.response || !(error.response.status >= 400 && error.response.status < 500)) {
        ctx.throw(502, 'Bad gateway.');
      }
      return await next();
    }
    
    if (!responseWebP || caching.has(cache.hash) || queue.has(ctx.path)) {
      return;
    }

    // Save the original file
    caching.add(cache.hash);
    
    const stream = fs.createWriteStream(source);
    response.data.pipe(stream);

    stream.on('error', () => {
      caching.delete(cache.hash);
    });

    stream.on('close', () => {
      caching.delete(cache.hash);
      // Create the WebP file
      if (!queue.has(ctx.path)) {
        queue.add(ctx.path);
      }
    });
  };
}