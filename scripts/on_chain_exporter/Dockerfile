FROM denoland/deno:latest

WORKDIR /app

# Prefer not to run as root.
USER deno

ENV PRUNTIME_EXPORTER_LISTEN_PORT=8011

# Cache the dependencies as a layer (the following two steps are re-run only when deps.ts is modified).
# Ideally cache deps.ts will download and compile _all_ external files used in main.ts.
ADD app.ts metrics.js .
RUN deno cache app.ts
RUN deno cache metrics.js

CMD ["run", "--allow-net", "--allow-env", "--allow-read", "app.ts"]
