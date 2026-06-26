cd /app && node_modules/.bin/prisma migrate deploy && node --import ./instrumentation.mjs docker-entry.js
