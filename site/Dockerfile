FROM node:22-alpine AS dependencies
WORKDIR /app
COPY package.json package-lock.json ./
COPY site/package.json ./site/package.json
RUN npm ci --workspace @merra/site

FROM node:22-alpine AS builder
WORKDIR /app
COPY --from=dependencies /app/node_modules ./node_modules
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
