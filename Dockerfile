FROM rust:1.67 as rust_builder
WORKDIR /usr/src/dino99
COPY . .
WORKDIR server
RUN cargo install --path .

FROM node:18 as frontend_builder
COPY . .
WORKDIR frontend
RUN npm ci && npm run build

FROM debian:bullseye-slim
COPY --from=rust_builder /usr/local/cargo/bin/server /usr/local/bin/server
COPY --from=frontend_builder /frontend/dist /dist
CMD ["server"]
EXPOSE 8080
