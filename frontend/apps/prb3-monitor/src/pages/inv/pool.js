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
import {Modal, ModalBody, ModalButton, ModalFooter, ModalHeader} from 'baseui/modal';
import {FormControl} from 'baseui/form-control';
import {Input} from 'baseui/input';
import * as yup from 'yup';
import {Checkbox} from 'baseui/checkbox';
import {useFormik} from 'formik';

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
  BooleanColumn({
    title: 'Enabled',
    mapDataToValue: (data) => data.enabled,
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
    reset?.();
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
                onClick: () => {
                  setModalOpen(true);
                  setCurrModalItem(null);
                },
              },
            ]}
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable
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
                    data: {pid},
                  },
                }) => {
                  if (confirm('Are you sure?')) {
                    try {
                      await rawFetcher({
                        url: '/wm/config',
                        method: 'POST',
                        data: {RemovePool: {pid}},
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
    pid: yup.number().positive().integer().required(),
    name: yup.string().required(),
    sync_only: yup.boolean(),
    disabled: yup.boolean(),
  })
  .required();

const InputModal = ({initialValue, isOpen, onClose}) => {
  const isEdit = !!initialValue;
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const formik = useFormik({
    validationSchema,
    initialValues: {
      pid: undefined,
      name: '',
      sync_only: false,
      disabled: false,
    },
    onSubmit: async (values) => {
      try {
        await rawFetcher({
          url: '/wm/config',
          method: 'POST',
          data: isEdit ? {UpdatePool: values} : {AddPool: values},
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
      pid: initialValue.pid,
      name: initialValue.name,
      sync_only: initialValue.sync_only,
      disabled: initialValue.disabled,
    });
  }, [initialValue]);
  const close = () => onClose(formik.resetForm);

  return (
    <Modal isOpen={isOpen} closeable={false} autoFocus onClose={close}>
      <form onSubmit={formik.handleSubmit}>
        <ModalHeader>{isEdit ? 'Edit Pool' : 'New Pool'}</ModalHeader>
        <ModalBody>
          <FormControl label="PID" error={formik.errors.pid || null}>
            <Input
              size="mini"
              name="pid"
              disabled={isEdit}
              onChange={formik.handleChange}
              value={formik.values.pid}
              type="number"
            />
          </FormControl>
          <FormControl label="Name" error={formik.errors.name || null}>
            <Input size="mini" name="name" onChange={formik.handleChange} value={formik.values.name} type="text" />
          </FormControl>
          <Checkbox name="disabled" onChange={formik.handleChange} checked={formik.values.disabled}>
            Disabled
          </Checkbox>
          <Checkbox name="sync_only" onChange={formik.handleChange} checked={formik.values.sync_only}>
            Sync Only
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
