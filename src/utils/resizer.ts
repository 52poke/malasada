import sharp, { ResizeOptions, WebpOptions } from 'sharp';
import { readFromS3, uploadToS3 } from './s3';

interface ResizerOptions {
    source: string;
    target: string;
    resize?: ResizeOptions;
    webp?: WebpOptions;
}

const resizer = ({source, target, resize, webp}: ResizerOptions): Promise<string> => {
    return new Promise((resolve, reject) => {
        const { headers, readStream } = readFromS3(source);
        readStream.on('error', reject);
        let streamResize = sharp();
        if (resize) {
            streamResize = streamResize.resize(resize);
        }
        if (webp) {
            streamResize = streamResize.webp(webp);
        }
        const pipe = readStream.pipe(streamResize);
        headers.then(headersData => {
            const { writeStream, promise } = uploadToS3(webp ? 'image/webp' : headersData['content-type'], target);
            pipe.pipe(writeStream);
            return promise;
        })
        .then(data => resolve(data.Key))
        .catch(reject);
    });
};

export default resizer;
