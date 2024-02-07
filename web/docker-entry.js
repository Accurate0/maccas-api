import { handler } from './build/handler.js';
import express from 'express';

const app = express();
app.disable('x-powered-by');

app.get('/health', (req, res) => {
	res.end('ok');
});

app.use(handler);

const server = app.listen(3000, () => {
	console.log('listening on port 3000');
});

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

function shutdown() {
	console.log('graceful shutdown express');
	server.close();
}
