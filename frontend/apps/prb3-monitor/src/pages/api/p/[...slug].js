export default async function handler(req, res) {
  const {slug} = req.query;
  const urlBase = slug.shift();
  const url = new URL(slug.join('/'), urlBase).href;

  const headers = req.headers;
  delete headers['content-length'];
  delete headers['Content-Length'];
  delete headers['transfer-encoding'];
  delete headers['Transfer-Encoding'];
  delete headers['content-type'];
  headers['Content-Type'] = 'application/json';

  const body = req.body
    ? (
      typeof req.body === 'object'
        ? JSON.stringify(req.body)
        : req.body
      )
    : undefined
  ;

  const r = await fetch(url, {
    method: req.method,
    cache: 'no-cache',
    headers,
    body,
  });
  res.status(r.status);
  res.end(await r.text());
}
