#!/bin/bash
set -e

# Start the backend API
/app/api &

# Start the Next.js frontend (serves on port 3000)
cd /app/frontend
npm start &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?