#!/usr/bin/env bash
set -e

DB_USER="milkfarm"
DB_PASS="milkfarm"
DB_NAME="milkfarm"

SEED_COWS="${SEED_COWS:-300}"
SEED_YEARS="${SEED_YEARS:-3}"

echo "Starting PostgreSQL..."
sudo systemctl start postgresql

echo "Ensuring database user and database exist..."
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1 || \
    sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw $DB_NAME || \
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

echo "Seeding mock data (${SEED_COWS} cows, ${SEED_YEARS} years)..."
(cd backend && cargo run --bin seed -- --cows "$SEED_COWS" --years "$SEED_YEARS")

cleanup() {
    echo ""
    echo "Stopping..."
    kill $BACKEND_PID $FRONTEND_PID 2>/dev/null
    wait $BACKEND_PID $FRONTEND_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM

echo "Starting backend..."
(cd backend && cargo run --bin milk-farm-backend) &
BACKEND_PID=$!

echo "Waiting for backend to be ready..."
until curl -sf http://localhost:3000/api/v1/health > /dev/null 2>&1; do
    sleep 1
done
echo "Backend is ready."

echo "Starting frontend..."
(cd frontend && npm run dev) &
FRONTEND_PID=$!

echo ""
echo "    Running!"
echo "    Frontend: http://localhost:5173"
echo "    Backend:  http://localhost:3000"
echo "    Login: admin / admin"
echo ""

wait
