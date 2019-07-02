import React from 'react';
import { observer } from 'mobx-react';

import AuthContainer from '../containers/AuthContainer';
import { RefreshableComponent, Button, ListInline } from './Utils';
import DataTable from './DataTable';

interface Props {
  auth: AuthContainer;
}

@observer
export default class Accounts extends RefreshableComponent<Props, {}> {
  refresh() {
    this.props.auth.refreshAccounts();
  }

  render() {
    return (
      <div>
        <DataTable
          title="Accounts"
          refresh={() => this.refresh()}
          rows={this.props.auth.accounts}
          headers={['Name', 'Public Key Base16']}
          renderRow={(account: Account) => {
            let keyBase16 = base64toHex(account.publicKey);
            return (
              <tr key={account.name}>
                <td>{account.name}</td>
                <td>{keyBase16}</td>
              </tr>
            );
          }}
          footerMessage={
            <span>
              You can create new account here; don't worry, the private key will
              never leave the browser, we'll save it straight to disk on your
              machine.
            </span>
          }
        />

        <ListInline>
          <Button title="Create account" onClick={() => this.createAccount()} />
        </ListInline>
      </div>
    );
  }

  createAccount() {
    alert('Create');
  }
}

function base64toHex(base64: string): string {
  const raw = atob(base64);
  let hex = '';
  for (let i = 0; i < raw.length; i++) {
    let _hex = raw.charCodeAt(i).toString(16);
    hex += _hex.length === 2 ? _hex : '0' + _hex;
  }
  return hex.toLowerCase();
}
