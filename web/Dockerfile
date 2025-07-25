FROM node:24-alpine AS builder
WORKDIR /app
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
ENV COREPACK_INTEGRITY_KEYS=0
RUN corepack enable

COPY . /app
WORKDIR /app

FROM builder AS build
RUN apk add openssl

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run db:generate
RUN pnpm run build

FROM node:24-alpine
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
ENV COREPACK_INTEGRITY_KEYS=0
RUN corepack enable
RUN apk add openssl

WORKDIR /app
COPY --from=build /app/build build/
COPY --from=build /app/node_modules node_modules/
COPY --from=build /app/docker-entry.js .
COPY --from=build /app/prisma prisma
COPY --from=build /app/run.sh .

COPY package.json .
EXPOSE 3000
ENV NODE_ENV=production

ENTRYPOINT [ "/bin/sh" ]
CMD ["/app/run.sh"]
