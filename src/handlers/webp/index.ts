import { APIGatewayProxyHandlerV2, APIGatewayProxyResultV2 } from 'aws-lambda';
import { existS3 } from '../../utils/s3';
import resizer from '../../utils/resizer';
import { serveS3, thumbRegex } from '../../utils';
import { HttpError } from 'http-errors';

export const handler: APIGatewayProxyHandlerV2 = async (event): Promise<APIGatewayProxyResultV2> => {
    if (!event.rawPath.match(/^\/webp\//)) {
        return {
            statusCode: 400,
            body: JSON.stringify({
                error: 'invalid request',
            }),
        };
    }
    const path = event.rawPath.substring(5);
    const key = decodeURIComponent(path.substring(1));
    try {
        if (!key.match(/\.(jpg|jpeg|gif|png|tif|tiff|bmp)$/i)) {
            return serveS3(key);
        }

        let target = `webp-cache/${key}`;
        const targetExists = await existS3(target);
        if (targetExists) {
            return serveS3(target);
        }

        const sourceExist = await existS3(key);
        if (sourceExist) {
            // only convert to webp
            target = await resizer({ source: key, target, webp: {} });
            return serveS3(target);
        }

        const match = path.match(thumbRegex);
        if (!match) {
            return {
                statusCode: 404,
                body: JSON.stringify({
                    error: 'object not found',
                }),
            };
        }
        // need to resize and convert to webp
        const fileName = decodeURIComponent(match[3]);
        const source = `wiki/${match[1] || ''}${match[2]}/${fileName}`;
        const width = parseInt(match[4], 10);
        target = await resizer({ source, target, resize: { width }, webp: {} });
        return serveS3(target);
    } catch (e) {
        return {
            statusCode: (e as HttpError).statusCode || 500,
            body: JSON.stringify({
                error: (e as HttpError).message || JSON.stringify(e),
            }),
        };
    }
}
