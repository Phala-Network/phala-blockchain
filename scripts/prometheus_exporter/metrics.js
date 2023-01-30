import { Gauge, Registry } from "https://deno.land/x/ts_prometheus@v0.3.0/mod.ts"
import { urlJoin } from 'https://deno.land/x/url_join@1.0.0/mod.ts'

const PRUNTIME_URL_BASE = Deno.env.get('PRUNTIME_URL_BASE') ?? 'http://127.0.0.1:8000'
const PRUNTIME_GET_INFO_URL = urlJoin(PRUNTIME_URL_BASE, '/get_info')
const PRUNTIME_GET_INFO_POLLING_INTERVAL = parseInt(Deno.env.get('PRUNTIME_GET_INFO_POLLING_INTERVAL') ?? '300')
const PRUNTIME_GET_INFO_POLLING_TIMEOUT = parseInt(Deno.env.get('PRUNTIME_GET_INFO_POLLING_TIMEOUT') ?? '200')

const sleep = (t) => new Promise(r => setTimeout(r, t))

export const registry = Registry.default

const wrapGauge = (name, help, getterFn) => {
  const gauge = Gauge.with({ name, help })
  gauge.set(0)
  return {
    gauge,
    applyInfo: (info) => gauge.set(getterFn(info))
  }
}
const gauges = [
  wrapGauge('blocknum', 'blocknum', info => info.blocknum),
  wrapGauge('headernum', 'headernum', info => info.headernum),
  wrapGauge('para_headernum', 'para_headernum', info => info.para_headernum),
  wrapGauge('score', 'score', info => info.score),
  wrapGauge('pending_messages', 'pending_messages', info => info.pending_messages),
  wrapGauge('rust_peak_used', 'rust_peak_used', info => info.memory_usage.rust_peak_used),
  wrapGauge('rust_used', 'rust_used', info => info.memory_usage.rust_used),
  wrapGauge('total_peak_used', 'total_peak_used', info => info.memory_usage.total_peak_used),
]

const getInfoLoop = async () => {
  console.info('Started polling pRuntime.', {
    PRUNTIME_URL_BASE,
    PRUNTIME_GET_INFO_URL,
    PRUNTIME_GET_INFO_POLLING_INTERVAL,
    PRUNTIME_GET_INFO_POLLING_TIMEOUT
  })
  while (true) {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), PRUNTIME_GET_INFO_POLLING_TIMEOUT)
    try {
      const res = await fetch(PRUNTIME_GET_INFO_URL, {
        signal: controller.timeout
      })
      if (res.ok) {
        const payload = JSON.parse((await res.json()).payload)
        gauges.forEach(g => g.applyInfo(payload))
      } else {
        console.error(`HTTP error! Status: ${res.status}`)
        gauges.forEach(g => g.gauge.set(0))
      }
    } catch (error) {
      console.error(error)
      gauges.forEach(g => g.gauge.set(0))
    } finally {
      clearTimeout(id)
    }
    await sleep(PRUNTIME_GET_INFO_POLLING_INTERVAL)
  }
}

export const getInfoLoopPromise = getInfoLoop()
