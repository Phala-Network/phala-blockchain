import React, {useCallback, useMemo} from 'react';
import {styled, useStyletron} from 'baseui';
import {StatefulDataTable, CategoricalColumn, NumericalColumn, StringColumn, COLUMNS} from 'baseui/data-table';
import {MobileHeader} from 'baseui/mobile-header';
import {TbAnalyze, TbCloudUpload, TbRefresh} from 'react-icons/tb';
import Head from 'next/head';
import {useAtomValue} from 'jotai';
import {currentFetcherAtom, currentWmAtom} from '@/state';
import useSWR from 'swr';
import {toaster} from 'baseui/toast';
import Column from 'baseui/data-table/column';
import {StringCell} from '@/utils';

const columns = [
  StringColumn({
    title: 'Name',
    mapDataToValue: (data) => data.worker.name,
    minWidth: 150,
  }),
  CategoricalColumn({
    title: 'PID',
    mapDataToValue: (data) => data.worker.pid.toString(),
    maxWidth: 60,
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
  CategoricalColumn({
    title: 'Session S.',
    mapDataToValue: (data) => {
      return data.session_info?.state || '';
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
      return (e ? `(From error state) ${e}` : '') + '\n' + (data.last_message || '');
    },
    textQueryFilter: function (textQuery, data) {
      return data.toLowerCase().includes(textQuery.toLowerCase());
    },
    sortable: false,
    filterable: false,
  }),
  NumericalColumn({
    title: 'Para Height',
    mapDataToValue: (data) => data.phactory_info?.blocknum || 0,
  }),
  NumericalColumn({
    title: 'Para Hd. Height',
    mapDataToValue: (data) => data.phactory_info?.para_headernum || 0,
  }),
  NumericalColumn({
    title: 'Relay Hd. Height',
    mapDataToValue: (data) => data.phactory_info?.headernum || 0,
  }),
  NumericalColumn({
    title: 'P Instant',
    mapDataToValue: (data) => data.session_info?.benchmark?.p_instant || 0,
  }),
  NumericalColumn({
    title: 'P Init',
    mapDataToValue: (data) => data.session_info?.benchmark?.p_init || 0,
  }),
  StringColumn({
    title: 'Public Key',
    mapDataToValue: (data) => data.phactory_info?.public_key || '',
  }),
  CategoricalColumn({
    title: 'pRuntime Version',
    mapDataToValue: (data) => data.phactory_info?.version || '',
  }),
  CategoricalColumn({
    title: 'pRuntime Git Rev.',
    mapDataToValue: (data) => data.phactory_info?.git_revision || '',
  }),
  NumericalColumn({
    title: 'rust_peak_used',
    mapDataToValue: (data) => data.phactory_info?.memory_usage.rust_peak_used || 0,
  }),
  NumericalColumn({
    title: 'rust_used',
    mapDataToValue: (data) => data.phactory_info?.memory_usage.rust_used || 0,
  }),
  NumericalColumn({
    title: 'total_peak_used',
    mapDataToValue: (data) => data.phactory_info?.memory_usage.total_peak_used || 0,
  }),
  StringColumn({
    title: 'UUID',
    mapDataToValue: (data) => data.worker.id || '',
  }),
];

const PageWrapper = styled('div', () => ({
  width: '100%',
  display: 'flex',
  flex: 1,
  flexFlow: 'column nowrap',
}));

export default function WorkerStatusPage() {
  const [css] = useStyletron();
  const currWm = useAtomValue(currentWmAtom);
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const fetcher = useCallback(async () => {
    const req = {
      url: '/workers/status',
    };
    const res = await rawFetcher(req);
    return res.data.workers.map((data) => ({
      data,
      id: data.worker.id,
    }));
  }, [rawFetcher]);
  const {data, isLoading, mutate} = useSWR(`worker_status_${currWm?.key}`, fetcher, {refreshInterval: 6000});

  const batchActions = useMemo(() => {
    return [
      {
        label: 'Restart',
        onClick: async ({selection}) => {
          if (!confirm('Are you sure?')) {
            return;
          }
          try {
            await rawFetcher({
              url: '/workers/restart',
              method: 'PUT',
              data: {ids: selection.map((i) => i.id)},
            });
            toaster.positive('Restarted.');
          } catch (e) {
            console.error(e);
            toaster.negative(e.toString());
          } finally {
            await mutate();
          }
        },
        renderIcon: () => (
          <span className={css({display: 'flex', alignItems: 'center', fontSize: '12px', lineHeight: '0'})}>
            <TbRefresh className={css({marginRight: '3px'})} size={16} />
            Restart
          </span>
        ),
      },

      {
        label: 'Force Register',
        onClick: async ({selection}) => {
          if (!confirm('Are you sure?')) {
            return;
          }
          try {
            await rawFetcher({
              url: '/workers/force_register',
              method: 'PUT',
              data: {ids: selection.map((i) => i.id)},
            });
            toaster.positive('Requested, check transaction status for progress.');
          } catch (e) {
            console.error(e);
            toaster.negative(e.toString());
          } finally {
            await mutate();
          }
        },
        renderIcon: () => (
          <span className={css({display: 'flex', alignItems: 'center', fontSize: '12px', lineHeight: '0'})}>
            <TbCloudUpload className={css({marginRight: '3px'})} size={16} />
            Force Register
          </span>
        ),
      },
    ];
  }, [css, mutate, rawFetcher]);
  const rowActions = useMemo(() => {
    return [
      {
        label: 'Restart',
        onClick: async ({row}) => {
          if (!confirm('Are you sure?')) {
            return;
          }
          try {
            await rawFetcher({
              url: '/workers/restart',
              method: 'PUT',
              data: {ids: [row.id]},
            });
            toaster.positive('Restarted.');
          } catch (e) {
            console.error(e);
            toaster.negative(e.toString());
          } finally {
            await mutate();
          }
        },
        renderIcon: () => (
          <span className={css({display: 'flex', alignItems: 'center', fontSize: '15px', lineHeight: '0'})}>
            <TbRefresh className={css({marginRight: '3px'})} size={16} />
            Restart
          </span>
        ),
      },

      {
        label: 'Force Register',
        onClick: async ({row}) => {
          if (!confirm('Are you sure?')) {
            return;
          }
          try {
            await rawFetcher({
              url: '/workers/force_register',
              method: 'PUT',
              data: {ids: [row.id]},
            });
            toaster.positive('Requested, check transaction status for progress.');
          } catch (e) {
            console.error(e);
            toaster.negative(e.toString());
          } finally {
            await mutate();
          }
        },
        renderIcon: () => (
          <span className={css({display: 'flex', alignItems: 'center', fontSize: '15px', lineHeight: '0'})}>
            <TbCloudUpload className={css({marginRight: '3px'})} size={16} />
            Force Register
          </span>
        ),
      },
    ];
  }, [css, mutate, rawFetcher]);

  const reloadWm = useCallback(async () => {
    if (!confirm('Are you sure?')) {
      return;
    }
    try {
      await rawFetcher({
        url: '/wm/restart',
        method: 'PUT',
      });
      toaster.positive('Restarted WM!');
    } catch (e) {
      toaster.negative(e.toString());
    } finally {
      await mutate();
    }
  }, [mutate, rawFetcher]);

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
            overrides={{
              Root: {
                style: () => ({
                  backgroundColor: 'transparent',
                }),
              },
            }}
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
              // {
              //   onClick: reloadWm,
              //   label: 'Restart WM',
              // },
            ]}
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable
            initialSortIndex={0}
            initialSortDirection="ASC"
            resizableColumnWidths
            columns={columns}
            rows={data || []}
            batchActions={batchActions}
            rowActions={rowActions}
          />
        </div>
      </PageWrapper>
    </>
  );
}
