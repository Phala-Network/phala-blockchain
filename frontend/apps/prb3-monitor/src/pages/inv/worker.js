import React, {useCallback, useEffect, useState} from 'react';
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
import {FiEdit, FiTrash2} from 'react-icons/fi';
import * as yup from 'yup';
import {useFormik} from 'formik';
import {Modal, ModalBody, ModalButton, ModalFooter, ModalHeader} from 'baseui/modal';
import {FormControl} from 'baseui/form-control';
import {Input} from 'baseui/input';
import {Checkbox} from 'baseui/checkbox';

const columns = [
  StringColumn({
    title: 'Name',
    mapDataToValue: (data) => data.name,
  }),
  CategoricalColumn({
    title: 'PID',
    mapDataToValue: (data) => data.pid.toString(),
  }),
  StringColumn({
    title: 'Endpoint',
    mapDataToValue: (data) => data.endpoint,
  }),
  StringColumn({
    title: 'Stake',
    mapDataToValue: (data) => data.stake,
  }),
  BooleanColumn({
    title: 'Enabled',
    mapDataToValue: (data) => data.enabled,
  }),
  BooleanColumn({
    title: 'GK mode',
    mapDataToValue: (data) => data.gatekeeper,
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

export default function WorkerInvPage() {
  const [css] = useStyletron();

  const currWm = useAtomValue(currentWmAtom);
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const fetcher = useCallback(async () => {
    const req = {
      url: '/wm/config',
      method: 'POST',
      data: {GetAllPoolsWithWorkers: null},
    };
    const res = await rawFetcher(req);
    return res.data
      .map((i) => i.workers)
      .flat()
      .map((data) => ({id: data.id, data}));
  }, [rawFetcher]);
  const {data, isLoading, mutate} = useSWR(`inv_workers_${currWm?.name}`, fetcher, {refreshInterval: 6000});
  const [currModalItem, setCurrModalItem] = useState(null);
  const [isModalOpen, setModalOpen] = useState(false);
  const onModalClose = (reset) => {
    setModalOpen(false);
    setCurrModalItem(null);
    reset?.();
    mutate();
  };

  return (
    <>
      <InputModal onClose={onModalClose} isOpen={isModalOpen} initialValue={currModalItem} />
      <Head>
        <title>{currWm ? currWm.name + ' - ' : ''}Worker Config</title>
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
            title={`Inventory - Workers (${data?.length || 0})`}
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
                onClick: () => {
                  setCurrModalItem(null);
                  setModalOpen(true);
                },
              },
            ]}
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable
            initialSortIndex={0}
            initialSortDirection="ASC"
            rowActions={[
              {
                renderIcon: () => <FiEdit />,
                onClick: ({row: {data}}) => {
                  setCurrModalItem({
                    ...data,
                    disabled: !data.enabled,
                  });
                  setModalOpen(true);
                },
              },
              {
                renderIcon: () => <FiTrash2 />,
                onClick: async ({
                  row: {
                    data: {name},
                  },
                }) => {
                  if (confirm('Are you sure?')) {
                    try {
                      await rawFetcher({
                        url: '/wm/config',
                        method: 'POST',
                        data: {RemoveWorker: {name}},
                      });
                    } catch (e) {
                      toaster.negative(e.response?.data?.message || e?.toString());
                    } finally {
                      await mutate();
                    }
                  }
                },
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

const validationSchema = yup
  .object({
    name: yup.string().required(),
    endpoint: yup.string().required(),
    stake: yup.number().positive().integer().required(),
    pid: yup.number().positive().integer().required(),
    sync_only: yup.boolean(),
    disabled: yup.boolean(),
    gatekeeper: yup.boolean(),
  })
  .required();

const InputModal = ({initialValue, isOpen, onClose}) => {
  const isEdit = !!initialValue;
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const formik = useFormik({
    validationSchema,
    initialValues: {
      name: '',
      endpoint: '',
      stake: undefined,
      pid: undefined,
      sync_only: false,
      disabled: false,
      gatekeeper: false,
    },
    onSubmit: async (values) => {
      try {
        await rawFetcher({
          url: '/wm/config',
          method: 'POST',
          data: isEdit
            ? {UpdateWorker: {...values, new_name: values.name, name: initialValue.name}}
            : {AddWorker: values},
        });
        toaster.positive('Success');
        onClose(formik.resetForm);
      } catch (e) {
        toaster.negative(e.response?.data?.message || e?.toString());
      }
    },
  });

  useEffect(() => {
    if (!initialValue) {
      return;
    }
    formik.setValues({
      name: initialValue.name,
      endpoint: initialValue.endpoint,
      stake: initialValue.stake,
      pid: initialValue.pid,
      sync_only: initialValue.sync_only,
      disabled: initialValue.disabled,
      gatekeeper: initialValue.gatekeeper,
    });
  }, [initialValue]);
  const close = () => onClose(formik.resetForm);

  return (
    <Modal isOpen={isOpen} closeable={false} autoFocus onClose={close}>
      <form onSubmit={formik.handleSubmit}>
        <ModalHeader>{isEdit ? `Edit Worker (${initialValue.name})` : 'New Worker'}</ModalHeader>
        <ModalBody>
          <FormControl label="Name" error={formik.errors.name || null}>
            <Input size="mini" name="name" onChange={formik.handleChange} value={formik.values.name} type="text" />
          </FormControl>
          <FormControl label="Endpoint" error={formik.errors.endpoint || null}>
            <Input
              size="mini"
              name="endpoint"
              onChange={formik.handleChange}
              value={formik.values.endpoint}
              type="text"
            />
          </FormControl>
          <FormControl label="Stake" error={formik.errors.stake || null}>
            <Input size="mini" name="stake" onChange={formik.handleChange} value={formik.values.stake} type="text" />
          </FormControl>
          <FormControl label="PID" error={formik.errors.pid || null}>
            <Input size="mini" name="pid" onChange={formik.handleChange} value={formik.values.pid} type="number" />
          </FormControl>
          <Checkbox name="disabled" onChange={formik.handleChange} checked={formik.values.disabled}>
            Disabled
          </Checkbox>
          <Checkbox name="sync_only" onChange={formik.handleChange} checked={formik.values.sync_only}>
            Sync Only
          </Checkbox>
          <Checkbox name="gatekeeper" onChange={formik.handleChange} checked={formik.values.gatekeeper}>
            Gatekeeper
          </Checkbox>
        </ModalBody>
        <ModalFooter>
          <ModalButton disabled={formik.isSubmitting} kind="tertiary" onClick={close}>
            Cancel
          </ModalButton>
          <ModalButton isLoading={formik.isSubmitting} disabled={formik.isSubmitting} type="submit">
            Submit
          </ModalButton>
        </ModalFooter>
      </form>
    </Modal>
  );
};
