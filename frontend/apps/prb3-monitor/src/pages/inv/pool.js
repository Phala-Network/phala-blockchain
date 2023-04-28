import React, {useCallback, useState} from 'react';
import {useStyletron} from 'baseui';
import {StatefulDataTable, CategoricalColumn, StringColumn, BooleanColumn} from 'baseui/data-table';
import {MobileHeader} from 'baseui/mobile-header';
import {TbAnalyze} from 'react-icons/tb';
import Head from 'next/head';
import {useAtomValue} from 'jotai';
import {currentFetcherAtom, currentWmAtom} from '@/state';
import useSWR from 'swr';
import {toaster} from 'baseui/toast';
import {PageWrapper} from '@/utils';
import {FiTrash2} from 'react-icons/fi';
import {Modal, ModalBody, ModalButton, ModalFooter, ModalHeader} from 'baseui/modal';

const columns = [
  CategoricalColumn({
    title: 'PID',
    mapDataToValue: (data) => data.pid.toString(),
  }),
  StringColumn({
    title: 'Name',
    mapDataToValue: (data) => data.name,
  }),

  BooleanColumn({
    title: 'Sync only mode',
    mapDataToValue: (data) => data.sync_only,
  }),
  StringColumn({
    title: 'UUID',
    mapDataToValue: (data) => data.id,
  }),
];

export default function PoolInvPage() {
  const [css] = useStyletron();
  const currWm = useAtomValue(currentWmAtom);
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const fetcher = useCallback(async () => {
    const req = {
      url: '/wm/config',
      method: 'POST',
      data: {GetAllPools: null},
    };
    const res = await rawFetcher(req);
    return res.data.map((data) => ({
      data,
      id: data.id,
    }));
  }, [rawFetcher]);
  const {data, isLoading, mutate} = useSWR(`inv_pool_${currWm?.name}`, fetcher, {refreshInterval: 6000});
  const [currModalItem, setCurrModalItem] = useState(null);
  const [isModalOpen, setModalOpen] = useState(false);
  const onModalClose = (reset) => {
    setModalOpen(false);
    setCurrModalItem(null);
    reset();
    mutate();
  };

  return (
    <>
      <InputModal onClose={onModalClose} isOpen={isModalOpen} initialValue={currModalItem} />
      <Head>
        <title>{currWm ? currWm.name + ' - ' : ''}Pool Config</title>
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
            title={`Inventory - Pools (${data?.length || 0})`}
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
                label: 'Add',
              },
            ]}
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable
            rowActions={[
              {
                renderIcon: () => <FiTrash2 />,
              },
            ]}
            resizableColumnWidths
            columns={columns}
            rows={data || []}
          />
        </div>
      </PageWrapper>
    </>
  );
}

const InputModal = ({initialValue, isOpen, onClose}) => {
  const [loading, setLoading] = useState(false);
  const reset = () => {};
  const close = () => onClose(reset);
  const submit = () => {};
  return (
    <Modal isOpen={isOpen} closeable={false} autoFocus onClose={close}>
      <ModalHeader>{initialValue ? `Edit Pool(${initialValue.id})` : 'New Pool'}</ModalHeader>
      <ModalBody></ModalBody>
      <ModalFooter>
        <ModalButton disabled={loading} kind="tertiary" onClick={close}>
          Cancel
        </ModalButton>
        <ModalButton isLoading={loading} onClick={submit()}>
          Submit
        </ModalButton>
      </ModalFooter>
    </Modal>
  );
};
