import crypto from 'crypto';
import path from 'path';
import fs from 'mz/fs';

export default class Cache {
  constructor(path, options) {
    this._path = path;
    this._hash = crypto.createHash('md5').update(path).digest('hex');
    this._options = options;
  }

  get path() {
    return this._path;
  }

  get hash() {
    return this._hash;
  }

  async source() {
    if (this._options.backendType === 'local') {
      return path.join(this._options.backend, this.path);
    }
    const tmpPath = path.join(this._options.cacheRootPath, 'tmp');
    if (!await fs.exists(tmpPath)) {
      await fs.mkdir(tmpPath);
    }
    return path.join(tmpPath, this.hash);
  }

  async directory() {
    const directory = path.join(this._options.cacheRootPath, this.hash.substr(0, 1), this.hash.substr(0, 2));

    if (!await fs.exists(this._options.cacheRootPath)) {
      await fs.mkdir(this._options.cacheRootPath);
    }
    if (!await fs.exists(path.join(this._options.cacheRootPath, this.hash.substr(0, 1)))) {
      await fs.mkdir(path.join(this._options.cacheRootPath, this.hash.substr(0, 1)));
    }
    if (!await fs.exists(directory)) {
      await fs.mkdir(directory);
    }
    
    return directory;
  }

  async tmpDest() {
    const directory = await this.directory();
    return path.join(directory, 'tmp_' + this.hash);
  }

  async dest() {
    const directory = await this.directory();
    return path.join(directory, this.hash + '.webp');
  }

  async purge() {
    const dest = await this.dest();
    if (await fs.exists(dest)) {
      await fs.unlink(dest);
      return true;
    }
    return false;
  }

  async cleanSource() {
    if (this._options.backendType !== 'local') {
      await fs.unlink(await this.source());
    }
  }
}