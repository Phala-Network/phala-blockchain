import React, {useCallback, useState} from 'react';
import {useStyletron} from 'baseui';
import {StatefulDataTable, CategoricalColumn, StringColumn} from 'baseui/data-table';
import {MobileHeader} from 'baseui/mobile-header';
import {TbAnalyze} from 'react-icons/tb';
import Head from 'next/head';
import {useAtomValue} from 'jotai';
import {currentFetcherAtom, currentWmAtom} from '@/state';
import useSWR from 'swr';
import {toaster} from 'baseui/toast';
import {PageWrapper} from '@/utils';
import {useFormik} from 'formik';
import {Modal, ModalBody, ModalButton, ModalFooter, ModalHeader} from 'baseui/modal';
import {FormControl} from 'baseui/form-control';
import {Input} from 'baseui/input';
import * as yup from 'yup';
import {Select} from 'baseui/select';

const columns = [
  CategoricalColumn({
    title: 'PID',
    mapDataToValue: (data) => data.pid.toString(),
  }),
  StringColumn({
    title: 'Operator Account Id',
    mapDataToValue: (data) => data.operator_account_id || '',
  }),
  StringColumn({
    title: 'Proxied Account Id',
    mapDataToValue: (data) => data.proxied_account_id || '',
  }),
];

export default function PoolOperatorInvPage() {
  const [css] = useStyletron();
  const currWm = useAtomValue(currentWmAtom);
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const fetcher = useCallback(async () => {
    const req = {
      url: '/wm/config',
      method: 'POST',
      data: {GetAllPoolOperators: null},
    };
    const res = await rawFetcher(req);
    return res.data.map((data) => ({
      data,
      id: data.pid,
    }));
  }, [rawFetcher]);
  const {data, isLoading, mutate} = useSWR(`inv_po_${currWm?.name}`, fetcher, {refreshInterval: 6000});
  const [isModalOpen, setModalOpen] = useState(false);
  const onModalClose = (reset) => {
    setModalOpen(false);
    reset?.();
    mutate();
  };

  return (
    <>
      <InputModal onClose={onModalClose} isOpen={isModalOpen} />
      <Head>
        <title>{currWm ? currWm.name + ' - ' : ''}Pool Operator Config</title>
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
            title={`Inventory - Pool Operators (${data?.length || 0})`}
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
                label: 'Set',
                onClick: () => {
                  setModalOpen(true);
                },
              },
            ]}
          />
          <div className={css({width: '12px'})} />
        </div>
        <div className={css({height: '100%', margin: '0 20px 20px'})}>
          <StatefulDataTable resizableColumnWidths columns={columns} rows={data || []} />
        </div>
      </PageWrapper>
    </>
  );
}

const validationSchema = yup
  .object({
    pid: yup.number().positive().integer().required(),
    account: yup.string().required(),
    account_type: yup.string().required(),
    proxied_account_id: yup.string(),
  })
  .required();

const InputModal = ({isOpen, onClose}) => {
  const rawFetcher = useAtomValue(currentFetcherAtom);
  const formik = useFormik({
    validationSchema,
    initialValues: {
      pid: undefined,
      account: '',
      account_type: 'Seed',
      proxied_account_id: '',
    },
    onSubmit: async (values) => {
      console.log(values);
      try {
        await rawFetcher({
          url: '/wm/config',
          method: 'POST',
          data: {
            SetPoolOperator: {
              ...values,
              proxied_account_id: values.proxied_account_id?.length ? values.proxied_account_id : undefined,
            },
          },
        });
        toaster.positive('Success');
        onClose(formik.resetForm);
      } catch (e) {
        toaster.negative(e.response?.data?.message || e?.toString());
      }
    },
  });

  const close = () => onClose(formik.resetForm);

  return (
    <Modal isOpen={isOpen} closeable={false} autoFocus onClose={close}>
      <form onSubmit={formik.handleSubmit}>
        <ModalHeader>Set pool operator</ModalHeader>
        <ModalBody>
          <FormControl label="PID" error={formik.errors.pid || null}>
            <Input size="mini" name="pid" onChange={formik.handleChange} value={formik.values.pid} type="number" />
          </FormControl>
          <FormControl label="Account Type" error={formik.errors.account_type || null}>
            <Select
              size="mini"
              name="account_type"
              onChange={({value: [selected]}) => {
                formik.setValues({...formik.values, account_type: selected ? selected.id : ''});
              }}
              value={
                formik.values.account_type ? [{label: formik.values.account_type, id: formik.values.account_type}] : []
              }
              options={[
                {label: 'Seed', id: 'Seed'},
                {label: 'SecretKey', id: 'SecretKey'},
              ]}
            />
          </FormControl>
          <FormControl label="Account" error={formik.errors.account || null}>
            <Input
              size="mini"
              name="account"
              onChange={formik.handleChange}
              value={formik.values.account}
              type="text"
            />
          </FormControl>
          <FormControl label="Proxied Account (SS58)" error={formik.errors.proxied_account_id || null}>
            <Input
              size="mini"
              name="proxied_account_id"
              onChange={formik.handleChange}
              value={formik.values.proxied_account_id}
              type="text"
            />
          </FormControl>
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
