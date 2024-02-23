import { IMAGE_CDN } from '$lib/constants';
import sharp from 'sharp';

export async function GET({ fetch, params: { imageBasename } }) {
	const response = await fetch(`${IMAGE_CDN}/${imageBasename}`, {
		credentials: 'omit',
		method: 'GET',
		referrer: ''
	});

	const image = await sharp(await response.arrayBuffer())
		.resize(180, 180)
		.jpeg({ mozjpeg: true })
		.toBuffer();

	const imageResponse = new Response(image, {
		headers: {
			'cache-control': 'max-age=604800',
			'content-type': 'image/jpeg'
		}
	});

	return imageResponse;
}
