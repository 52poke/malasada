import { APIGatewayProxyHandler, APIGatewayProxyResult } from 'aws-lambda';
import { existS3 } from '../../utils/s3';
import resizer from '../../utils/resizer';
import { serveS3, thumbRegex } from '../../utils';

export const handler: APIGatewayProxyHandler = async (event): Promise<APIGatewayProxyResult> => {
    if (!event.path.match(/^\/webp\//)) {
        return {
            statusCode: 400,
            body: JSON.stringify({
                error: 'invalid request',
            }),
        };
    }
    const path = event.path.substr(5);
    const key = decodeURIComponent(path.substr(1));
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
            statusCode: e.statusCode || 500,
            body: JSON.stringify({
                error: e.message || JSON.stringify(e),
            }),
        };
    }
}
