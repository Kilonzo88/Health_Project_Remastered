#!/bin/sh
# wait-for-services.sh

set -e

host_mongo="$1"
shift
host_ipfs="$1"
shift

# Wait for MongoDB with credentials
until mongosh "mongodb://admin:password@$host_mongo:27017/healthcare?authSource=admin" --eval "db.adminCommand('ping')" --quiet > /dev/null 2>&1; do
  >&2 echo "MongoDB is unavailable - sleeping"
  sleep 1
done

>&2 echo "MongoDB connection successful - checking initialization..."

# Wait for initialization (with timeout)
MAX_RETRIES=30
RETRY_COUNT=0
until mongosh "mongodb://admin:password@$host_mongo:27017/healthcare?authSource=admin" --eval "db.meta.findOne({_id: 'ready'})" --quiet > /dev/null 2>&1 || [ $RETRY_COUNT -eq $MAX_RETRIES ]; do
  >&2 echo "MongoDB initialization not complete - sleeping (attempt $RETRY_COUNT/$MAX_RETRIES)"
  sleep 2
  RETRY_COUNT=$((RETRY_COUNT + 1))
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
  >&2 echo "Warning: MongoDB initialization check timed out, proceeding anyway..."
fi

>&2 echo "MongoDB is up"

# Wait for IPFS
until curl -X POST http://"$host_ipfs":5001/api/v0/version > /dev/null 2>&1; do
  >&2 echo "IPFS is unavailable - sleeping"
  sleep 1
done

>&2 echo "IPFS is up - executing command"

exec "$@"
