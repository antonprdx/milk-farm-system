#!/usr/bin/env bash
set -e

DB_USER="milkfarm"
DB_PASS="milkfarm"
DB_NAME="milkfarm"

SEED_COWS="${SEED_COWS:-100}"
SEED_YEARS="${SEED_YEARS:-1}"

echo "Starting PostgreSQL..."
sudo systemctl start postgresql

echo "Ensuring database user and database exist..."
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1 || \
    sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw $DB_NAME || \
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

echo "Seeding mock data (${SEED_COWS} cows, ${SEED_YEARS} years)..."
(cd backend && cargo run --bin seed -- --cows "$SEED_COWS" --years "$SEED_YEARS")

ML_PORT=8001
ML_VENV="analytics-ml/.venv"

echo "Starting ML service..."
if [ ! -d "$ML_VENV" ]; then
    echo "  Creating Python venv for analytics-ml..."
    python3 -m venv "$ML_VENV"
    "$ML_VENV/bin/pip" install -q --upgrade pip
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 "$ML_VENV/bin/pip" install -q -r analytics-ml/requirements.txt
fi
DATABASE_URL="postgresql+asyncpg://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME" \
    "$ML_VENV/bin/uvicorn" app.main:app --host 0.0.0.0 --port $ML_PORT --app-dir analytics-ml &
ML_PID=$!

cleanup() {
    echo ""
    echo "Stopping..."
    kill $BACKEND_PID $FRONTEND_PID $ML_PID 2>/dev/null
    wait $BACKEND_PID $FRONTEND_PID $ML_PID 2>/dev/null
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
echo "    Frontend:  http://localhost:5173"
echo "    Backend:   http://localhost:3000"
echo "    ML service: http://localhost:$ML_PORT"
echo "    Login: admin / admin"
echo ""

wait
