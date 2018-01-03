import Koa from 'koa';
import fs from 'mz/fs';
import program from 'commander';
import middleware from './lib/middleware';
import defaultPath from './lib/defaultPath';

program
  .option('-c, --config [file]', 'Set the configuration JSON file.')
  .parse(process.argv);

let config = {};
if (program.config || fs.existsSync(defaultPath.configPath)) {
  config = JSON.parse(fs.readFileSync(program.config || defaultPath.configPath, { encoding: 'utf-8' }));
}

const app = new Koa();

app.use(middleware(config));

export const server = app.listen(process.env.PORT || config.port || 3002, function () {
  console.log('Malasada server listening on port ' + server.address().port);
});