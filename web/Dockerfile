FROM node:20-alpine AS builder
WORKDIR /app
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

COPY . /app
WORKDIR /app

FROM builder AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run db:generate
RUN pnpm run build

FROM node:20-alpine

WORKDIR /app
COPY --from=build /app/build build/
COPY --from=build /app/node_modules node_modules/

COPY package.json .
EXPOSE 3000
ENV NODE_ENV=production
CMD [ "node", "build" ]
