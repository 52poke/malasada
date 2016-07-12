import CWebp from 'cwebp';
import crypto from 'crypto';
import fs from 'mz/fs';
import path from 'path';
import _ from 'lodash';

const cacheRootPath = path.join(__dirname, '..', 'cache');
const tmpPath = path.join(cacheRootPath, 'tmp');

export default class Queue {
  constructor(maxPending) {
    this.queue = [];
    this.pending = 0;
    this.maxPending = maxPending || 1;
  }

  add(path) {
    this.queue.push({path: path});
    this._dequeue();
  }

  has(path) {
    return !!_.find(this.queue, {path: path});
  }

  async _dequeue() {
    const item = this.queue.shift();

    if (this.pending >= this.maxPending || !item) {
      return false;
    }

    try {
      this.pending++;
      const hash = crypto.createHash('md5').update(item.path).digest('hex');
      const directory = path.join(cacheRootPath, hash.substr(0, 1), hash.substr(0, 2));

      if (!await fs.exists(path.join(cacheRootPath, hash.substr(0, 1)))) {
        await fs.mkdir(path.join(cacheRootPath, hash.substr(0, 1)));
      }

      if (!await fs.exists(directory)) {
        await fs.mkdir(directory);
      }
      
      const encoder = new CWebp(path.join(tmpPath, hash));
      await encoder.write(path.join(directory, 'tmp_' + hash));
      await fs.rename(path.join(directory, 'tmp_' + hash), path.join(directory, hash + '.webp'));
      await fs.unlink(path.join(tmpPath, hash));

      this.pending--;
      this._dequeue();

    } catch (error) {
      console.log(error);
      this.pending--;
      this._dequeue();
    }

    return true;
  }
}