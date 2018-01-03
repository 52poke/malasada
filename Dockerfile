FROM node:alpine

RUN mkdir -p /app
WORKDIR /app

COPY package.json /app/
COPY package-lock.json /app/

RUN apk update && apk add --update bash curl
RUN curl -s https://raw.githubusercontent.com/Intervox/node-webp/latest/bin/install_webp | bash
RUN npm install && npm cache clean --force

COPY . /app
ENV NODE_ENV production

EXPOSE 3002
VOLUME ["/app/config.json", "/app/cache"]

CMD [ "npm", "start" ]