import { Readable } from 'stream';
import concat from 'concat-stream';
import { APIGatewayProxyResult } from 'aws-lambda';
import { readFromS3 } from './s3';

export const thumbRegex = /^\/wiki\/thumb\/(archive\/)?([0-9a-f]\/[0-9a-f][0-9a-f])\/([^\/]+)\/([0-9]+)px-(.*)$/;

const streamToBuffer = (
    headers: Promise<{[key: string]: string}>,
    readStream: Readable,
): Promise<{ headersData: {[key: string]: string},  buffer: Buffer }> => {
    let headersData: {[key: string]: string} = {};
    headers.then(data => {
        headersData = data;
    });
    
    return new Promise((resolve, reject) => {
        const writable = concat(buffer => {
            resolve({
                buffer,
                headersData,
            });
        });
        writable.on('error', reject);
        readStream.on('error', reject);
        readStream.pipe(writable);
    });
}

export const serveS3 = async (key: string): Promise<APIGatewayProxyResult> => {
    const { headers, readStream } = readFromS3(key);
    const { headersData, buffer } = await streamToBuffer(headers, readStream);
    return {
        statusCode: 200,
        body: buffer.toString('base64'),
        isBase64Encoded: true,
        headers: {
            'Content-Type': headersData['content-type'],
        },
    };
}
