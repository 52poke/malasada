import { APIGatewayProxyHandler, APIGatewayProxyResult } from 'aws-lambda';
import resizer from '../../utils/resizer';
import { serveS3, thumbRegex } from '../../utils';
import { existS3 } from '../../utils/s3';

export const handler: APIGatewayProxyHandler = async (event): Promise<APIGatewayProxyResult> => {
    const match = event.path.match(thumbRegex);
    if (!match || match[3] !== match[5]) {
        return {
            statusCode: 400,
            body: JSON.stringify({
                error: 'invalid request',
            }),
        };
    }
    const fileName = decodeURIComponent(match[3]);
    const source = `wiki/${match[1] || ''}${match[2]}/${fileName}`;
    const width = parseInt(match[4], 10);
    let target = `wiki/thumb/${match[1] || ''}${match[2]}/${fileName}/${width}px-${fileName}`;

    try {
        const targetExists = await existS3(target);
        if (!targetExists) {
            target = await resizer({ source, target, resize: { width } });
        }
        return serveS3(target);
    } catch (e) {
        return {
            statusCode: e.statusCode || 500,
            body: JSON.stringify({
                error: e.message || JSON.stringify(e),
            }),
        };
    }
};
