import { IMAGE_CDN } from '$lib/constants';

export async function GET({ fetch, params: { imageBasename } }) {
	const response = await fetch(`${IMAGE_CDN}/${imageBasename}`, {
		credentials: 'omit',
		method: 'GET',
		referrer: ''
	});

	const imageResponse = new Response(response.body, {
		headers: {
			'cache-control': 'max-age=604800',
			'content-type': response.headers.get('content-type') ?? ''
		}
	});

	return imageResponse;
}
