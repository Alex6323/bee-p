#!/bin/bash


USER="$1"
DB_NAME="$2"
CLEANUP_FILE="cleanup.sql"


echo "DROP DATABASE "$DB_NAME";" > $CLEANUP_FILE

psql -U postgres -f $CLEANUP_FILE

rm -f CLEANUP_FILE
