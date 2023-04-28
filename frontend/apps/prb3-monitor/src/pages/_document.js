import * as React from 'react';
import Document, {Head, Html, Main, NextScript} from 'next/document';
import {Provider as StyletronProvider} from 'styletron-react';
import {styletron} from '@/styletron';

class MyDocument extends Document {
  static async getInitialProps(props) {
    // eslint-disable-next-line react/display-name
    const page = await props.renderPage((App) => (props) => {
      return (
        <StyletronProvider value={styletron}>
          <App {...props} />
        </StyletronProvider>
      );
    });
    const stylesheets = styletron.getStylesheets() || [];
    return {...page, stylesheets};
  }

  render() {
    return (
      <Html>
        <Head>
          {this.props.stylesheets.map((sheet, i) => (
            <style
              className="_styletron_hydrate_"
              dangerouslySetInnerHTML={{__html: sheet.css}}
              media={sheet.attrs.media}
              data-hydrate={sheet.attrs['data-hydrate']}
              key={i}
            />
          ))}
        </Head>
        <body>
          <Main />
          <NextScript />
        </body>
      </Html>
    );
  }
}

export default MyDocument;
