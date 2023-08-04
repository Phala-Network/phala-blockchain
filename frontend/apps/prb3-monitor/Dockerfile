FROM node:18-bullseye-slim as builder
ARG DEBIAN_FRONTEND=noninteractive

WORKDIR /app
RUN --mount=type=cache,target=/root/.yarn yarn global add next

COPY package.json .
COPY yarn.lock .

RUN --mount=type=cache,target=/root/.yarn yarn install

COPY . .
RUN yarn next build

FROM node:18-bullseye-slim

WORKDIR /app
ENV NODE_ENV production
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static
USER nextjs

CMD ["node", "server.js"]