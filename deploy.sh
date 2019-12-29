#!/bin/sh

PATH=./node_modules/.bin:$PATH

sls config credentials \
  --provider aws \
  --key ${SLS_KEY} \
  --secret ${SLS_SECRET}

echo 'Deploying'
sls deploy --stage ${STAGE} --bucket ${BUCKET} --region ${REGION}
