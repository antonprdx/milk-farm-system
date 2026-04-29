#!/bin/bash
set -e

echo "Running migrations..."
./seed --cows ${SEED_COWS:-300} --years ${SEED_YEARS:-3}

echo "Starting backend..."
exec ./backend
