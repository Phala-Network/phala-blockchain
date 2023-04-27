export default async function handler(req, res) {
  const {slug} = req.query;
  const urlBase = slug.shift();
  const url = new URL(slug.join('/'), urlBase).href;

  const headers = req.headers;
  delete headers['content-length'];
  delete headers['Content-Length'];
  const r = await fetch(url, {
    method: req.method,
    cache: 'no-cache',
    headers,
    body: req.body ? (typeof req.body === 'object' ? JSON.stringify(req.body) : req.body) : undefined,
  });
  res.status(r.status);
  res.end(await r.text());
}
