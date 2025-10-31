#!/bin/sh
# wait-for-mongo.sh

set -e

host="$1"
shift

until mongosh --host "$host" --eval "db.getSiblingDB('healthcare').getCollectionNames().includes('meta') && db.getSiblingDB('healthcare').getCollection('meta').findOne({key: 'ready'})" --quiet; do
  >&2 echo "MongoDB is unavailable - sleeping"
  sleep 1
done

>&2 echo "MongoDB is up - executing command"
exec "$@"
