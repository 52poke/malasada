const path = require('path');
const cacheRootPath = path.join(__dirname, '..', 'cache');
const configPath = path.join(__dirname, '..', 'config.json');

module.exports = {
  cacheRootPath,
  configPath
};