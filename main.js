#!/usr/bin/env node

require = require('@std/esm')(module); // eslint-disable-line no-global-assign
module.exports = require('./main.mjs').default;
