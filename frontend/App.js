import "regenerator-runtime/runtime";
import React, { useState } from "react";

import "./assets/css/global.css";

import {
  login,
  logout,
  get_greeting,
  set_greeting,
  get_bikes,
  return_bike,
  inspect_bike,
  ft_balance_of,
  storage_balance_of,
  storage_deposit,
  ft_transfer,
  ft_transfer_call,
} from "./assets/js/near/utils";

export default function App() {
  // use React Hooks to store greeting in component state
  const [greeting, setGreeting] = React.useState();

  // when the user has not yet interacted with the form, disable the button
  const [buttonDisabled, setButtonDisabled] = React.useState(true);

  // after submitting the form, we want to show Notification
  const [showNotification, setShowNotification] = React.useState(false);

  const [bikes, setBikes] = useState([]);

  // トランザクションの処理中を扱うフラグ
  const [inProcess, setInProcess] = useState(false);

  // ユーザがストレージを登録しているかを扱うフラグ
  const [storageRegistered, setStorageRegistered] = useState(false);

  // 残高表示をするアカウント名
  const [accountToShowBalance, setAccountToShowBalance] = useState("");

  // 表示する残高
  const [showBalance, setShowBalance] = useState(0);

  // 初回レンダリング時の処理
  // サイン後はページがリロードされるので,サインをする度に初回レンダリングで実行される
  React.useEffect(() => {
    // get_greeting is in near/utils.js
    get_greeting().then((greetingFromContract) => {
      setGreeting(greetingFromContract);
    });
    // bikeの情報を取得
    get_bikes().then((bikesFromContract) => {
      setBikes(bikesFromContract);
    });
    // ユーザのアカウントがFTコントラクトに登録されているかを確認
    // 空文字列(:ユーザがサインイン前)はエラーを引き起こすので条件式
    if (window.accountId) {
      // ストレージ残高にnullが返ってくる場合は未登録を意味する
      storage_balance_of(window.accountId).then((balance) => {
        console.log("user's storage balance: ", balance);
        if (balance === null) {
          setStorageRegistered(false);
        } else {
          setStorageRegistered(true);
        }
      });
    }
  }, []);

  //storage_depositの呼び出し
  const storageDeposit = async () => {
    try {
      storage_deposit().then((value) => {
        console.log("Returnd value from storage_deposit: ", value);
      });
    } catch (e) {
      alert(e);
    }
  };

  // ft_trasnfer_callを呼ぶことでBikeコントラクトにFT送金 + Bikeを使用
  const trasferFtToUseBike = async (index) => {
    console.log("Use bike");
    // 余分なトランザクションを避けるためにユーザの残高を確認
    let user_balance = await ft_balance_of(window.accountId);
    if (user_balance < 30) {
      alert("Balance is not enough");
      return;
    } else {
      try {
        ft_transfer_call(index);
      } catch (e) {
        alert(e);
      }
    }
  };

  const inspectThenGetBikes = async (index) => {
    console.log("Inspect bike");
    setInProcess(true);
    try {
      await inspect_bike(index);
    } catch (e) {
      alert(e);
    }
    get_bikes().then((bikesFromContract) => {
      setBikes(bikesFromContract);
    });
    setInProcess(false);
  };

  const returnThenGetBikes = async (index) => {
    console.log("Return bike");
    setInProcess(true);
    try {
      await return_bike(index);
    } catch (e) {
      alert(e);
    }
    get_bikes().then((bikesFromContract) => {
      setBikes(bikesFromContract);
    });
    setInProcess(false);
  };

  const ftTransfer = async () => {
    console.log("call transfer");
    // TODO: awaitつけるとおこらた
    // error中身 -> ReferenceError: await is not defined
    try {
      ft_transfer();
    } catch (e) {
      alert(e);
    }
  };

  const getThenSetBalance = async (account_id) => {
    let user_balance = await ft_balance_of(account_id);
    setShowBalance(user_balance);
    setAccountToShowBalance(account_id);
  };

  // if not signed in, return early with sign-in prompt
  if (!window.walletConnection.isSignedIn()) {
    return (
      <main>
        <h1>
          <label
            htmlFor="greeting"
            style={{
              color: "var(--secondary)",
              borderBottom: "2px solid var(--secondary)",
            }}
          >
            {greeting}
          </label>
          ! Welcome to NEAR!
        </h1>
        <p>
          Your contract is storing a greeting message in the NEAR blockchain. To
          change it you need to sign in using the NEAR Wallet. It is very
          simple, just use the button below.
        </p>
        <p>
          Do not worry, this app runs in the test network ("testnet"). It works
          just like the main network ("mainnet"), but using NEAR Tokens that are
          only for testing!
        </p>
        <p style={{ textAlign: "center", marginTop: "2.5em" }}>
          <button onClick={login}>Sign in</button>
        </p>
      </main>
    );
  }

  if (!storageRegistered) {
    console.log("user is not yet registered");
    return (
      <main>
        <h1>
          <label
            htmlFor="greeting"
            style={{
              color: "var(--secondary)",
              borderBottom: "2px solid var(--secondary)",
            }}
          >
            {greeting}
          </label>
          ! Welcome to NEAR!
        </h1>
        <p>
          Your contract is storing a greeting message in the NEAR blockchain. To
          change it you need to sign in using the NEAR Wallet. It is very
          simple, just use the button below.
        </p>
        <p>
          Do not worry, this app runs in the test network ("testnet"). It works
          just like the main network ("mainnet"), but using NEAR Tokens that are
          only for testing!
        </p>
        <p style={{ textAlign: "center", marginTop: "2.5em" }}>
          <button onClick={storageDeposit}>storage deposit</button>
        </p>
      </main>
    );
  }

  return (
    // use React Fragment, <>, to avoid wrapping elements in unnecessary divs
    <>
      <button className="link" style={{ float: "right" }} onClick={logout}>
        Sign out
      </button>
      <main>
        <h1>
          <label
            htmlFor="greeting"
            style={{
              color: "var(--secondary)",
              borderBottom: "2px solid var(--secondary)",
            }}
          >
            {greeting}
          </label>
          {
            " " /* React trims whitespace around tags; insert literal space character when needed */
          }
          {window.accountId}!
        </h1>
        <form
          onSubmit={async (event) => {
            event.preventDefault();

            // get elements from the form using their id attribute
            const { fieldset, greeting } = event.target.elements;

            // hold onto new user-entered value from React's SynthenticEvent for use after `await` call
            const newGreeting = greeting.value;

            // disable the form while the value gets updated on-chain
            fieldset.disabled = true;

            try {
              // make an update call to the smart contract
              // pass the value that the user entered in the greeting field
              await set_greeting(newGreeting);
            } catch (e) {
              alert(
                "Something went wrong! " +
                  "Maybe you need to sign out and back in? " +
                  "Check your browser console for more info."
              );
              throw e;
            } finally {
              // re-enable the form, whether the call succeeded or failed
              fieldset.disabled = false;
            }

            // update local `greeting` variable to match persisted value
            setGreeting(newGreeting);

            // show Notification
            setShowNotification(true);

            // remove Notification again after css animation completes
            // this allows it to be shown again next time the form is submitted
            setTimeout(() => {
              setShowNotification(false);
            }, 11000);
          }}
        >
          <fieldset id="fieldset">
            <label
              htmlFor="greeting"
              style={{
                display: "block",
                color: "var(--gray)",
                marginBottom: "0.5em",
              }}
            >
              Change greeting
            </label>
            <div style={{ display: "flex" }}>
              <input
                autoComplete="off"
                defaultValue={greeting}
                id="greeting"
                onChange={(e) => setButtonDisabled(e.target.value === greeting)}
                style={{ flex: 1 }}
              />
              <button
                disabled={buttonDisabled}
                style={{ borderRadius: "0 5px 5px 0" }}
              >
                Save
              </button>
            </div>
          </fieldset>
        </form>
        {inProcess === true ? (
          <p> in process... </p>
        ) : (
          bikes.map((bike, index) => {
            return (
              <div style={{ display: "flex" }}>
                {index}: bike
                <button
                  disabled={!bike.available}
                  onClick={() => trasferFtToUseBike(index)}
                  style={{ borderRadius: "5px 5px 5px 5px" }}
                >
                  use
                </button>
                <button
                  disabled={!bike.available}
                  onClick={() => inspectThenGetBikes(index)}
                  style={{ borderRadius: "5px 5px 5px 5px" }}
                >
                  inspect
                </button>
                <button
                  disabled={
                    !(
                      (bike.in_use && bike.used_by === window.accountId) ||
                      (bike.inspection &&
                        bike.inspected_by === window.accountId)
                    )
                  }
                  onClick={() => returnThenGetBikes(index)}
                  style={{ borderRadius: "5px 5px 5px 5px" }}
                >
                  return
                </button>
              </div>
            );
          })
        )}
        <button onClick={() => getThenSetBalance(window.accountId)}>
          show my balance
        </button>
        <button onClick={() => getThenSetBalance(process.env.CONTRACT_NAME)}>
          ft_balance_of_bike_contract
        </button>
        <button onClick={ftTransfer}>transfer</button>
        {accountToShowBalance && (
          <p>
            {accountToShowBalance}'s balance: {showBalance}
          </p>
        )}
      </main>
    </>
  );
}
