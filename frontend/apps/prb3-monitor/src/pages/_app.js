import React, {useEffect} from 'react';
import App from 'next/app';
import {Provider as StyletronProvider} from 'styletron-react';
import {LightTheme, BaseProvider, styled, useStyletron} from 'baseui';
import {styletron} from '@/styletron';
import Nav from '@/components/Nav';
import '@/styles/globals.css';
import {Provider, useAtomValue} from 'jotai';
import {allWmAtomRaw} from '@/state';
import {Modal, ModalBody, ModalHeader} from 'baseui/modal';
import {Spinner} from 'baseui/spinner';
import {SWRConfig} from 'swr';
import {toaster, ToasterContainer} from 'baseui/toast';
import {PLACEMENT} from 'baseui/tooltip';

const Wrapper = styled('div', () => ({
  display: 'flex',
  flexFlow: 'column nowrap',
  height: '100vh',
  overflow: 'hidden',
}));

const Inner = styled('div', () => ({
  padding: '9px 0 0',
  flex: 1,
  display: 'flex',
  overflowY: 'auto',
}));

const AppWrapper = ({children}) => {
  const {state, error} = useAtomValue(allWmAtomRaw);
  const [css, _] = useStyletron();
  useEffect(() => {
    if (error) {
      console.error('AppWrapper:', error);
    }
  }, [error]);
  return (
    <>
      {state === 'hasData' ? (
        <Wrapper>
          <ToasterContainer autoHideDuration={9000} placement={PLACEMENT.topRight}></ToasterContainer>
          <SWRConfig
            value={{
              onError: (error, key) => {
                console.error(key, error);
                toaster.negative(error.toString());
              },
            }}
          >
            {children}
          </SWRConfig>
        </Wrapper>
      ) : null}
      <Modal closeable={false} isOpen={state !== 'hasData'}>
        {state === 'loading' ? (
          <ModalBody>
            <div className={css({display: 'flex', alignItems: 'center'})}>
              <Spinner $size="21px" $borderWidth="3px" $color="black" />
              <p
                className={css({
                  fontSize: '15px',
                  fontWeight: 'bold',
                  color: 'black',
                  margin: '0 12px',
                })}
              >
                Loading
              </p>
            </div>
          </ModalBody>
        ) : state === 'hasError' ? (
          <>
            <ModalHeader>Error while initializing monitor</ModalHeader>
            <ModalBody>
              <h4>Please check browser console for detail</h4>
              <pre
                className={css({
                  whiteSpace: 'pre-line',
                })}
              >
                {error.toString()}
              </pre>
            </ModalBody>
          </>
        ) : null}
      </Modal>
    </>
  );
};
export default class MyApp extends App {
  render() {
    const {Component, pageProps} = this.props;
    return (
      <StyletronProvider value={styletron}>
        <BaseProvider theme={LightTheme}>
          <Provider>
            <AppWrapper>
              <Nav />
              <Inner>
                <Component {...pageProps} />
              </Inner>
            </AppWrapper>
          </Provider>
        </BaseProvider>
      </StyletronProvider>
    );
  }
}
