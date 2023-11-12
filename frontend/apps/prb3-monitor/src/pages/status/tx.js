import React, {useCallback} from 'react';
import {useStyletron} from 'baseui';
import {StatefulDataTable, CategoricalColumn, NumericalColumn, StringColumn, COLUMNS} from 'baseui/data-table';
import {MobileHeader} from 'baseui/mobile-header';
import {TbAnalyze} from 'react-icons/tb';
import Head from 'next/head';
import {useAtomValue} from 'jotai';
import {currentFetcherAtom, currentWmAtom} from '@/state';
import useSWR from 'swr';
import {toaster} from 'baseui/toast';
import {PageWrapper, StringCell} from '@/utils';
import Column from 'baseui/data-table/column';

const columns = [
  NumericalColumn({
    title: 'ID',
    mapDataToValue: (data) => data.id || '',
  }),
  CategoricalColumn({
    title: 'PID',
    mapDataToValue: (data) => data.pid.toString(),
  }),
  CategoricalColumn({
    title: 'State',
    mapDataToValue: (data) => {
      const s = data?.state;
      if (!s) {
        return 'Unknown';
      }
      if (typeof s === 'string') {
        return s;
      }
      return Object.keys(s)[0];
    },
  }),

  Column({
    kind: COLUMNS.STRING,
    title: 'Message',
    maxWidth: 720,
    minWidth: 450,
    buildFilter: function (params) {
      return function (data) {
        return true;
      };
    },
    renderCell: (props) => {
      return (
        <StringCell
          {...props}
          style={{cursor: 'zoom-in'}}
          onClick={() => alert(JSON.stringify(JSON.parse(props.value), null, 4))}
        />
      );
    },
    mapDataToValue: (data) => {
      const s = data.state;
      if (!s) {
        return '';
      }
      if (typeof s === 'string') {
        return s;
      }
      return JSON.stringify(Object.values(s)[0]);
    },
    textQueryFilter: function (textQuery, data) {
      return data.toLowerCase().includes(textQuery.toLowerCase());
    },
    sortable: false,
    filterable: false,
  }),
  StringColumn({
    title: 'Desc.',
    mapDataToValue: (data) => data.desc,
  }),
  StringColumn({
    title: 'Created at',
    mapDataToValue: (data) => data.created_at,
  }),
];

export default function WorkerStatusPage() {
  const [css] = useStyletron();
  const currWm = useAtomValue(currentWmAtom);
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const fetcher = useCallback(async () => {
    const req = {
      url: '/tx/status',
    };
    const res = await rawFetcher(req);
    const ret = res.data;
    ret.txs = [...ret.running_txs, ...ret.pending_txs, ...ret.past_txs].map((data) => ({data, id: data.id}));
    return ret;
  }, [rawFetcher]);
  const {data, isLoading, mutate} = useSWR(`tx_status_${currWm?.name}`, fetcher, {refreshInterval: 6000});

  return (
    <>
      <Head>
        <title>{currWm ? currWm.name + ' - ' : ''}Transaction Status</title>
      </Head>
      <PageWrapper>
        <div
          className={css({
            width: '100%',
            flex: 1,
            marginRight: '24px',
            display: 'flex',
          })}
        >
          <MobileHeader
            overrides={{
              Root: {
                style: () => ({
                  backgroundColor: 'transparent',
                }),
              },
            }}
            title={`Transactions (${data?.tx_count || 0})`}
            navButton={
              isLoading
                ? {
                    renderIcon: () => <TbAnalyze size={24} className="spin" />,
                    onClick: () => {},
                    label: 'Loading',
                  }
                : {
                    renderIcon: () => <TbAnalyze size={24} />,
                    onClick: () => {
                      mutate().then(() => toaster.positive('Reloaded'));
                    },
                    label: 'Reload',
                  }
            }
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable
            initialSortIndex={0}
            initialSortDirection="ASC"
            resizableColumnWidths
            columns={columns}
            rows={data?.txs || []}
          />
        </div>
      </PageWrapper>
    </>
  );
}
