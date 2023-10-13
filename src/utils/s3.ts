import stream, { Readable } from 'stream';
import AWS, { S3, AWSError } from 'aws-sdk';
import createHttpError from 'http-errors';

const s3 = new S3({
    credentials: process.env.S3_ACCESS_KEY_ID && process.env.S3_SECRET_ACCESS_KEY ? {
        accessKeyId: process.env.S3_ACCESS_KEY_ID,
        secretAccessKey: process.env.S3_SECRET_ACCESS_KEY,
    } : undefined,
    endpoint: process.env.S3_ENDPOINT,
    region: process.env.S3_REGION,
    s3ForcePathStyle: true,
});
const bucket = process.env.S3_BUCKET as string;

export const existS3 = async (key: string): Promise<boolean> => {
    try {
        await s3.headObject({ Bucket: bucket, Key: key }).promise();
        return true;
    } catch (err) {
        const error = err as AWSError;
        if (error.code === 'NoSuchKey' || error.code === 'Forbidden' || error.code === 'NotFound' || error.code === 'AccessDenied' || error.statusCode === 403 || error.statusCode === 404) {
            return false;
        }
        throw err;
    }
}

export const readFromS3 = (source: string): {
    headers: Promise<{[key: string]: string}>,
    readStream: Readable
} => {
    let readStream: Readable | undefined;
    const promise: Promise<{[key: string]: string}> = new Promise((resolve, reject) => {
        readStream = s3.getObject({ Bucket: bucket,  Key: source })
            .on('httpHeaders', (statusCode, headers, response, statusMessage) => {
                if (statusCode < 200 || statusCode >= 300) {
                    return reject(createHttpError(statusCode, statusMessage));
                }
                resolve(headers);
            })
            .createReadStream();
    });
    return {
        readStream: readStream as Readable,
        headers: promise,
    };
}

export const uploadToS3 = (contentType: string, target: string): {
    promise: Promise<S3.ManagedUpload.SendData>,
    writeStream: stream.Writable,
} => {
    const writeStream = new stream.PassThrough();
    const promise = s3.upload({
        ContentType: contentType,
        Bucket: bucket,
        Key: target,
        Body: writeStream,
    }).promise();
    return { writeStream, promise };
}

export const deleteS3 = (key: string): Promise<S3.DeleteObjectOutput> => {
    return s3.deleteObject({ Bucket: bucket, Key: key }).promise();
}
