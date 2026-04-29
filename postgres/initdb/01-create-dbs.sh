#!/bin/bash
set -e

psql -v ON_ERROR_STOP=0 --username "$POSTGRES_USER" -d "$POSTGRES_DB" <<-EOSQL
    SELECT 'CREATE DATABASE milk_farm_mlflow' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'milk_farm_mlflow')\gexec
    GRANT ALL PRIVILEGES ON DATABASE milk_farm_mlflow TO $POSTGRES_USER;
EOSQL
