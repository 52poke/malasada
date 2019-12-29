import { APIGatewayProxyHandler, APIGatewayProxyResult } from 'aws-lambda';
import { deleteS3, existS3 } from '../../utils/s3';

export const handler: APIGatewayProxyHandler = async (event): Promise<APIGatewayProxyResult> => {
    if (!event.path.match(/^\/webp\//)) {
        return {
            statusCode: 400,
            body: 'invalid request',
        };
    }
    const path = event.path.substr(5);
    const key = decodeURIComponent(path.substr(1));
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
            statusCode: e.statusCode || 500,
            body: JSON.stringify({
                error: e.message || JSON.stringify(e),
            }),
        };
    }
}
