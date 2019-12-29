FROM lambci/lambda:build-nodejs12.x
COPY . .
RUN npm install

RUN ["chmod", "+x", "deploy.sh"]
CMD ./deploy.sh