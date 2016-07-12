import Koa from 'koa';
import path from 'path';
import axios from 'axios';
import _ from 'lodash';
import config from './config.json';

const app = new Koa();

app.use(async (ctx, next) => {
  console.log(ctx.path);
  const response = await axios.get(ctx.path, {
    baseURL: config.baseURL,
    headers: {'Host': config.host},
    responseType: 'stream'
  });

  _.each(response.headers, (value, key) => {
    ctx.set(key, value);
  });
  ctx.body = response.data;
});

const server = app.listen(process.env.PORT || 3002, function() {
  console.log('Paradise server listening on port ' + server.address().port);
});