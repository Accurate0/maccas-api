import { handler } from './build/handler.js';
import express from 'express';

const app = express();

app.get('/health', (req, res) => {
	res.end('ok');
});

app.use(handler);
app.use(function (req, res, next) {
	res.removeHeader('Server');
	next();
});

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

function shutdown() {
	console.log('graceful shutdown express');
	app.close(function () {
		console.log('closed express');
	});
}

app.listen(3000, () => {
	console.log('listening on port 3000');
});
