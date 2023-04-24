import {atom} from 'jotai';
import {atomWithStorage, loadable} from 'jotai/utils';
import {parse} from 'yaml';

export const allWmAtomRaw = loadable(
  atom(async () => {
    const res = await fetch('/wm.yml');
    const yml = await res.text();
    return parse(yml) || [];
  }),
);

export const allWmAtom = atom((get) => {
  const r = get(allWmAtomRaw);
  return r.data || [];
});

export const allWmAtomInObject = atom((get) => {
  const ret = {};
  for (const i of get(allWmAtom)) {
    ret[i.name] = i;
  }
  return ret;
});

export const currentWmIdAtom = atomWithStorage('currentWmIdAtom', '');

export const currentWmAtom = atom((get) => {
  const currentWmId = get(currentWmIdAtom);
  const o = get(allWmAtomInObject);
  const ret = currentWmId?.length ? o[currentWmId] : get(allWmAtom)[0];
  ret.urlBase = ret.proxied ? `/api/p/${encodeURIComponent(ret.endpoint)}/` : new URL('./', ret.endpoint).href;
  return ret;
});

export const currentUrlAtom__worker_status = atom((get) => {
  const wm = get(currentWmAtom);
  if (!wm) {
    return '';
  }
  return wm.proxied ? wm.urlBase + 'workers/status' : new URL('./workers/status', wm.urlBase);
});

export const currentUrlAtom__tx_status = atom((get) => {
  const wm = get(currentWmAtom);
  if (!wm) {
    return '';
  }
  return wm.proxied ? wm.urlBase + 'tx/status' : new URL('./tx/status', wm.urlBase);
});
