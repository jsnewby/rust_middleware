FROM node:10.15.1-alpine as frontend-build
WORKDIR /app
RUN apk add make gcc g++ python git
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build


FROM node:10.15.1-alpine as frontend
WORKDIR /app
COPY --from=frontend-build /app/.nuxt /app/.nuxt
RUN npm install nuxt@2.8.1 @nuxtjs/axios@5.5.4 @download/blockies@1.0.3 clipboard-copy@3.0.0 bignumber.js@9.0.0 && \
    npm cache clean --force
COPY package.json package.json
COPY LICENSE.md LICENSE.md
ENV HOST 0.0.0.0
EXPOSE 80
CMD [ "npm", "start" ]