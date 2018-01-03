FROM node:alpine

RUN mkdir -p /app
WORKDIR /app

COPY package.json /app/
COPY package-lock.json /app/

RUN apk update && apk add --update libwebp-tools
RUN npm install && npm cache clean --force

COPY . /app
ENV NODE_ENV production

EXPOSE 3002
VOLUME ["/app/config.json", "/app/cache"]

CMD [ "npm", "start" ]