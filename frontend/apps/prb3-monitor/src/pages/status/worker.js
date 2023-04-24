import React, {useMemo} from 'react';
import {styled, useStyletron} from 'baseui';
import {StatefulDataTable, CategoricalColumn, NumericalColumn, StringColumn, COLUMNS} from 'baseui/data-table';
import {MobileHeader} from 'baseui/mobile-header';
import {TbAnalyze, TbCloudUpload, TbRefresh} from 'react-icons/tb';
import Head from 'next/head';
import {useAtomValue} from 'jotai';
import {currentUrlAtom__worker_status, currentWmAtom} from '@/state';
import useSWR from 'swr';
import {toaster} from 'baseui/toast';
import Column from 'baseui/data-table/column';
import {StringCell} from '@/utils';

const columns = [
  StringColumn({
    title: 'Name',
    mapDataToValue: (data) => data.worker.name,
  }),
  CategoricalColumn({
    title: 'PID',
    mapDataToValue: (data) => data?.worker.pid.toString(),
  }),
  CategoricalColumn({
    title: 'Status',
    mapDataToValue: (data) => {
      const s = data?.state;
      if (!s) {
        return;
      }
      if (typeof s === 'string') {
        return s;
      }
      return Object.keys(s)[0];
    },
  }),
  Column({
    kind: COLUMNS.STRING,
    title: 'Last Message',
    maxWidth: 720,
    minWidth: 450,
    buildFilter: function (params) {
      return function (data) {
        return true;
      };
    },
    renderCell: (props) => {
      return <StringCell {...props} style={{cursor: 'zoom-in'}} onClick={() => alert(props.value)} />;
    },
    mapDataToValue: (data) => {
      const s = data?.state;
      const e = s?.HasError;
      return (e ? `(From error state) ${e}` : '') + '\n' + data.last_message;
    },
    textQueryFilter: function (textQuery, data) {
      return data.toLowerCase().includes(textQuery.toLowerCase());
    },
    sortable: false,
    filterable: false,
  }),
  NumericalColumn({
    title: 'Para Height',
    mapDataToValue: (data) => data.phactory_info?.blocknum,
  }),
  NumericalColumn({
    title: 'Para Hd. Height',
    mapDataToValue: (data) => data.phactory_info?.para_headernum,
  }),
  NumericalColumn({
    title: 'Relay Hd. Height',
    mapDataToValue: (data) => data.phactory_info?.headernum,
  }),
  StringColumn({
    title: 'Public Key',
    mapDataToValue: (data) => data.phactory_info?.public_key,
  }),
  CategoricalColumn({
    title: 'pRuntime Version',
    mapDataToValue: (data) => data.phactory_info?.version,
  }),
  CategoricalColumn({
    title: 'pRuntime Git Rev.',
    mapDataToValue: (data) => data.phactory_info?.git_revision,
  }),
  NumericalColumn({
    title: 'rust_peak_used',
    mapDataToValue: (data) => data.phactory_info?.memory_usage.rust_peak_used,
  }),
  NumericalColumn({
    title: 'rust_used',
    mapDataToValue: (data) => data.phactory_info?.memory_usage.rust_used,
  }),
  NumericalColumn({
    title: 'total_peak_used',
    mapDataToValue: (data) => data.phactory_info?.memory_usage.total_peak_used,
  }),
  StringColumn({
    title: 'UUID',
    mapDataToValue: (data) => data.worker.id,
  }),
];

const PageWrapper = styled('div', () => ({
  width: '100%',
  display: 'flex',
  flex: 1,
  flexFlow: 'column nowrap',
}));

const fetcher = async (url) => {
  if (!url) {
    return [];
  }
  return (await fetch(url).then((r) => r.json())).workers.map((data) => ({
    data,
    id: data.worker.id,
  }));
};
export default function WorkerStatusPage() {
  const [css] = useStyletron();
  const currWm = useAtomValue(currentWmAtom);
  const url = useAtomValue(currentUrlAtom__worker_status);
  const {data, isLoading, mutate} = useSWR(url, fetcher, {refreshInterval: 6000});

  const actions = useMemo(() => {
    return [
      {
        label: 'Restart',
        onClick: ({row}) => alert('todo'),
        renderIcon: () => (
          <span className={css({display: 'flex', alignItems: 'center', fontSize: '12px', lineHeight: '0'})}>
            <TbRefresh className={css({marginRight: '3px'})} size={16} />
            Restart
          </span>
        ),
      },

      {
        label: 'Re-register',
        onClick: ({row}) => alert('todo'),
        renderIcon: () => (
          <span className={css({display: 'flex', alignItems: 'center', fontSize: '12px', lineHeight: '0'})}>
            <TbCloudUpload className={css({marginRight: '3px'})} size={16} />
            Re-register
          </span>
        ),
      },
    ];
  }, [css]);
  return (
    <>
      <Head>
        <title>{currWm ? currWm.name + ' - ' : ''}Worker Status</title>
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
            title={`Workers (${data ? data.length : 0})`}
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
            actionButtons={[
              {
                onClick: () => {},
                label: 'Restart All',
              },
            ]}
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable
            resizableColumnWidths
            columns={columns}
            rows={data || []}
            batchActions={actions}
            rowActions={actions}
          />
        </div>
      </PageWrapper>
    </>
  );
}
