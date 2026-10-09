#!/bin/sh
api=$(cat ../runners-project-api/VERSION)
frontend=$(cat ../runners-project-frontend/VERSION)
if [ "$api" != "$frontend" ]; then
  printf '../runners-project-frontend/VERSION\tfrontend version %s differs from api version %s\n' "$frontend" "$api"
  exit 1
fi
