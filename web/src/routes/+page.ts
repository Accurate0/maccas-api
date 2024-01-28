import { redirect } from '@sveltejs/kit';

export const load = () => {
	redirect(302, '/offers'); // needs `throw` in v1
};
