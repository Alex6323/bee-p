#!/bin/bash

#usage:
#./setup.sh schema.sql username password dbname

PATH_TO_SCHEMA="$1"
USER="$2"
PASS="$3"
DB_NAME="bee"


if [ -z "$1" ]
  then
    echo "Need to provide path to schema arg (first positional arg)"
fi


if [ -z "$2" ]
  then
    echo "Need to provide db's username (second positional arg)"
fi

if [ -z "$3" ]
  then
    echo "Need to provide username's password (third positional arg)"
fi

if [ -z "$4" ]
  then
    echo "using default db name: $DB_NAME"
  else
    DB_NAME="$4"
fi

export BEE_DATABASE_URL="postgres://$USER:$PASS@localhost/$DB_NAME"


createdb -U $USER $DB_NAME
psql -f $PATH_TO_SCHEMA -U $USER $DB_NAME