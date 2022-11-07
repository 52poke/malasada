import { APIGatewayProxyHandlerV2, APIGatewayProxyResultV2 } from 'aws-lambda';
import { HttpError } from 'http-errors';
import { deleteS3, existS3 } from '../../utils/s3';

export const handler: APIGatewayProxyHandlerV2 = async (event): Promise<APIGatewayProxyResultV2> => {
    if (!event.rawPath.match(/^\/webp\//)) {
        return {
            statusCode: 400,
            body: 'invalid request',
        };
    }
    const path = event.rawPath.substring(5);
    const key = decodeURIComponent(path.substring(1));
    try {
        const target = `webp-cache/${key}`;
        const targetExists = await existS3(target);
        if (!targetExists) {
            return {
                statusCode: 404,
                body: 'webp cache not found',
            };
        }
        await deleteS3(target);
        return {
            statusCode: 204,
            body: '',
        };
    } catch (e) {
        return {
            statusCode: (e as HttpError).statusCode || 500,
            body: JSON.stringify({
                error: (e as HttpError).message || JSON.stringify(e),
            }),
        };
    }
}
