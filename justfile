dev:
  cargo watch -x run
build-frontend:
  cd client && npm run build
  rm -rf ./dist/
  cp -r client/dist/ ./dist/

