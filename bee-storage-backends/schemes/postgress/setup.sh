#!/bin/bash

#TODO - move to readme
#Please edit the postgres configuration file to trust connection
#via unix sockets:
#edit the conf:
#sudo nano /etc/postgresql/11/main/pg_hba.conf
#To look like (replace peer with "trust"):

## Database administrative login by Unix domain socket
#local   all             postgres                                trust
# ....
# "local" is for Unix domain socket connections only
#local   all             all                                     trust

#Then restart service (replace '11' with your current version):
#service postgresql@11-main restart


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

sed -i '/BEE_DATABASE_URL/d' ~/.bashrc
echo "'"$PASS"'"
echo  export BEE_DATABASE_URL="\"postgres://"$USER":"$PASS"@localhost/"$DB_NAME"\"" >> ~/.bashrc
source ~/.bashrc


echo "CREATE USER "$USER" LOGIN PASSWORD '"$PASS"';" >> setup.sql
echo "CREATE DATABASE "$DB_NAME" WITH OWNER = "$USER";" >> setup.sql
echo "GRANT ALL PRIVILEGES ON DATABASE "$DB_NAME" TO "$USER";" >> setup.sql


sudo -u postgres psql -f setup.sql
sudo -u postgres psql -f $PATH_TO_SCHEMA -U $USER $DB_NAME

rm -f setup.sql