import {atom} from 'jotai';
import {atomWithStorage, loadable} from 'jotai/utils';
import {parse} from 'yaml';
import axios from 'axios';

const isDev = process.env.NODE_ENV === 'development';

export const allWmAtomRaw = loadable(
  atom(async () => {
    const res = await fetch(isDev ? '/wm.dev.yml' : '/wm.yml');
    const yml = await res.text();
    return parse(yml) || [];
  }),
);

export const allWmAtom = atom((get) => {
  const r = get(allWmAtomRaw);
  if (!r.data) {
      return [];
  }

  for (const wm of r.data) {
    wm.key = wm.name.toLowerCase()
      .replaceAll(/[\s]+/g, '-')
      .replaceAll(/[^a-z0-9-]/g, '')
    ;
  }

  return r.data;
});

export const allWmAtomInObject = atom((get) => {
  const ret = {};
  for (const i of get(allWmAtom)) {
    ret[i.key] = i;
  }
  return ret;
});

export const currentWmIdAtom = atom((get) => {
  const allWm = get(allWmAtom);
  const wmid = document?.location.pathname.split('/')[1];
  return wmid;
});

export const currentWmAtom = atom((get) => {
  const currentWmId = get(currentWmIdAtom);
  const o = get(allWmAtomInObject);
  return currentWmId?.length ? o[currentWmId] : get(allWmAtom)[0];
});

export const currentAxios = atom((get) => {
  const wm = get(currentWmAtom);
  if (!wm) {
    return null;
  }
  return axios.create({
    baseURL: wm.proxied ? `/api/p/${encodeURIComponent(wm.endpoint)}/` : wm.endpoint,
    timeout: 12000,
  });
});

export const currentFetcherAtom = atom((get) => {
  return async (req) => {
    if (!req) {
      return null;
    }
    const axios = get(currentAxios);
    if (!axios) {
      return null;
    }
    return axios.request(req);
  };
});
