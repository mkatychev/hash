FROM node:16.18.1-alpine AS builder

WORKDIR /app

# Ensure that the node module layer can be cached
COPY package.json .
COPY yarn.lock .
RUN yarn install --frozen-lockfile --production --ignore-scripts --prefer-offline

# Also ensure that api node modules can be cached
COPY libs/javascript/@local/tsconfig/package.json libs/javascript/@local/tsconfig/
COPY libs/javascript/@local/eslint-config/package.json libs/javascript/@local/eslint-config/
COPY packages/hash/backend-utils/package.json packages/hash/backend-utils/
COPY packages/hash/realtime/package.json packages/hash/realtime/
RUN yarn workspace @hashintel/hash-realtime install --frozen-lockfile --ignore-scripts --prefer-offline

COPY libs/javascript/@local/tsconfig libs/javascript/@local/tsconfig
COPY libs/javascript/@local/eslint-config libs/javascript/@local/eslint-config
COPY packages/hash/backend-utils packages/hash/backend-utils
COPY packages/hash/realtime packages/hash/realtime

#########################################################################################

FROM node:16.18.1-alpine

# Run as a non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
COPY --from=builder --chown=appgroup:appuser /app /app

WORKDIR /app

USER appuser
ENV NODE_ENV production

CMD ["yarn", "workspace","@hashintel/hash-realtime", "start"]

