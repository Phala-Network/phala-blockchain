import { Application, Router } from "https://deno.land/x/oak@v11.1.0/mod.ts";
import { getInfoLoopPromise, registry } from "./metrics.js";

const PRUNTIME_EXPORTER_LISTEN_PORT = parseInt(Deno.env.get('PRUNTIME_EXPORTER_LISTEN_PORT') ?? '8080')

const app = new Application()

const router = new Router()
router
  .get("/metrics", (ctx) => {
    ctx.response.headers.set("Content-Type", "")
    ctx.response.body = registry.metrics()
  });

app.use(router.routes())

const startServer = async () => {
  console.info(`Listening on port ${PRUNTIME_EXPORTER_LISTEN_PORT}...`)
  await app.listen({ port: PRUNTIME_EXPORTER_LISTEN_PORT })
}

await Promise.all([
  getInfoLoopPromise,
  startServer()
])
