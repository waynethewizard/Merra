FROM node:22-alpine AS dependencies
WORKDIR /app/site
COPY site/package.json site/package-lock.json ./
RUN npm ci

FROM node:22-alpine AS builder
WORKDIR /app
COPY --from=dependencies /app/site/node_modules ./site/node_modules
COPY site ./site
COPY Cargo.toml ./Cargo.toml
COPY docs ./docs
COPY golden ./golden
COPY scenarios ./scenarios
WORKDIR /app/site
RUN npm run build

FROM caddy:2.10-alpine AS runtime
COPY site/Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/site/out /srv
EXPOSE 8080
