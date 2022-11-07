import { APIGatewayProxyHandlerV2, APIGatewayProxyResultV2 } from 'aws-lambda';
import resizer from '../../utils/resizer';
import { serveS3, thumbRegex } from '../../utils';
import { existS3 } from '../../utils/s3';
import { HttpError } from 'http-errors';

export const handler: APIGatewayProxyHandlerV2 = async (event): Promise<APIGatewayProxyResultV2> => {
    const match = event.rawPath.match(thumbRegex);
    if (!match) {
        return {
            statusCode: 400,
            body: JSON.stringify({
                error: 'invalid request',
            }),
        };
    }
    const fileName = decodeURIComponent(match[3]);
    const targetFileName = decodeURIComponent(match[5]);
    const source = `wiki/${match[1] || ''}${match[2]}/${fileName}`;
    const width = parseInt(match[4], 10);
    let target = `wiki/thumb/${match[1] || ''}${match[2]}/${fileName}/${width}px-${targetFileName}`;

    try {
        const targetExists = await existS3(target);
        if (!targetExists) {
            target = await resizer({ source, target, resize: { width } });
        }
        return serveS3(target);
    } catch (e) {
        return {
            statusCode: (e as HttpError).statusCode || 500,
            body: JSON.stringify({
                error: (e as HttpError).message || JSON.stringify(e),
            }),
        };
    }
};
