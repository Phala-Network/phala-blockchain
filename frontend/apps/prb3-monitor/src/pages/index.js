import Head from 'next/head';
import {useRouter} from 'next/router';
import {useEffect} from 'react';

export default function Home() {
  const r = useRouter();
  useEffect(() => {
    r.replace('/status/worker');
  }, [r]);
  return (
    <>
      <Head>
        <title>Monitor</title>
      </Head>
    </>
  );
}
