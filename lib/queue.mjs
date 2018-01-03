import CWebp from 'cwebp';
import fs from 'mz/fs';
import _ from 'lodash';
import Cache from './cache';

export default class Queue {
  constructor(options) {
    this.queue = [];
    this.pending = 0;
    this._options = options;
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

    if (this.pending >= this._options.queue || !item) {
      return false;
    }

    this.pending++;
    const cache = new Cache(item.path, this._options);
    const source = await cache.source();

    try {
      const tmpDest = await cache.tmpDest();
      const dest = await cache.dest();
      const encoder = new CWebp(source);
      await encoder.write(tmpDest);
      await fs.rename(tmpDest, dest);
      await cache.cleanSource();
      this.pending--;
      this._dequeue();
    } catch (error) {
      await cache.cleanSource();
      this.pending--;
      this._dequeue();
    }

    return true;
  }
}