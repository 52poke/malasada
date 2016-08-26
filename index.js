import Koa from 'koa';
import path from 'path';
import crypto from 'crypto';
import axios from 'axios';
import _ from 'lodash';
import send from 'koa-send';
import fs from 'mz/fs';
import normalizeHeaderCase from 'header-case-normalizer';
import config from './config.json';
import Queue from './lib/queue';

const app = new Koa();
const queue = new Queue();

const cacheRootPath = path.join(__dirname, 'cache');
const tmpPath = path.join(cacheRootPath, 'tmp');
const writing = [];

app.use(async (ctx, next) => {
  if (ctx.path.indexOf('/purge/') === 0) {
    const hash = crypto.createHash('md5').update(ctx.path.substr(6)).digest('hex');
    const filepath = path.join(hash.substr(0, 1), hash.substr(0, 2), hash + '.webp');
    if (await fs.exists(path.join(cacheRootPath, filepath))) {
      await fs.unlink(path.join(cacheRootPath, filepath));
    }
    ctx.body = null;
    return;
  }
  
  // Send the cached WebP file
  const hash = crypto.createHash('md5').update(ctx.path).digest('hex');
  const filepath = path.join(hash.substr(0, 1), hash.substr(0, 2), hash + '.webp');
  if (await fs.exists(path.join(cacheRootPath, filepath))) {
    await send(ctx, filepath, {root: cacheRootPath, maxAge: 3628800 * 1000});
    return;
  }

  if (!await fs.exists(tmpPath)) {
    await fs.mkdir(tmpPath);
  }

  // Send the original file and convert to WebP
  const response = await axios.get(ctx.path, {
    baseURL: config.baseURL,
    headers: {'Host': config.host},
    responseType: 'stream'
  });

  _.each(response.headers, (value, key) => {
    ctx.set(normalizeHeaderCase(key), value);
  });
  ctx.set('Cache-Control', 'private');

  ctx.body = response.data;

  if (writing.includes(hash) || queue.has(ctx.path)) {
    return;
  }

  // Save the original file
  const stream = fs.createWriteStream(path.join(tmpPath, hash));
  response.data.pipe(stream);

  stream.on('error', () => {
    _.remove(writing, hash);
  });

  stream.on('close', () => {
    _.remove(writing, hash);
    // Create the WebP file
    if (!queue.has(ctx.path)) {
      queue.add(ctx.path);
    }
  });
});

const server = app.listen(process.env.PORT || 3002, function () {
  console.log('Paradise server listening on port ' + server.address().port);
});