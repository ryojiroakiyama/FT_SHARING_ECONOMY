import "regenerator-runtime/runtime";
import React, { useState } from "react";

import "./assets/css/global.css";

import {
  login,
  logout,
  return_bike,
  inspect_bike,
  ft_balance_of,
  storage_balance_of,
  storage_deposit,
  ft_transfer,
  ft_transfer_call,
  is_available,
  who_is_using,
  who_is_inspecting,
  num_of_bikes,
} from "./assets/js/near/utils";

export default function App() {
  const [bikes, setBikes] = useState([]);

  // トランザクションの処理中を扱うフラグ
  const [inProcess, setInProcess] = useState(false);

  // ユーザがストレージを登録しているかを扱うフラグ
  const [storageRegistered, setStorageRegistered] = useState(false);

  // 残高表示をするアカウント名
  const [accountToShowBalance, setAccountToShowBalance] = useState("");

  // 表示する残高
  const [showBalance, setShowBalance] = useState(0);

  const createBikes = async () => {
    let num = await num_of_bikes();
    console.log("Num of bikes:", num);
    let bikes = [];
    for (let i = 0; i < num; i++) {
      let field = { available: false, in_use: false, inspection: false };
      field.available = await is_available(i);
      await who_is_using(i).then((user_id) => {
        if (window.accountId === user_id) {
          field.in_use = true;
        }
      });
      await who_is_inspecting(i).then((inspector) => {
        if (window.accountId === inspector) {
          field.inspection = true;
        }
      });
      console.log("before push: ", i, ":", field);
      bikes.push(field);
    }
    setBikes(bikes);
    console.log("set bikes: ", bikes);
  };

  // ストレージ残高にnullが返ってくる場合は未登録を意味する
  const isStorageRegistered = async (account_id) => {
    let balance = await storage_balance_of(account_id);
    console.log("user's storage balance: ", balance);
    if (balance === null) {
      console.log("account is not yet registered");
      return false;
    } else {
      return true;
    }
  };

  // consol.error

  // 初回レンダリング時の処理
  // サイン後はページがリロードされるので,サインをする度に初回レンダリングで実行される
  React.useEffect(() => {
    // bikeの情報を取得
    createBikes();

    // ユーザのアカウントがFTコントラクトに登録されているかを確認
    const checkStorageRegistered = async (account_id) => {
      const is = await isStorageRegistered(account_id);
      setStorageRegistered(is);
    };
    // 空文字列(:ユーザがサインイン前)はエラーを引き起こすので条件式
    if (window.accountId) {
      checkStorageRegistered(window.accountId);
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
    await createBikes();
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
    await createBikes();
    setInProcess(false);
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
        <p style={{ textAlign: "center", marginTop: "2.5em" }}>
          <button onClick={login}>Sign in</button>
        </p>
      </main>
    );
  }

  // if not storage registered, return early with storage register prompt
  if (!storageRegistered) {
    return (
      <main>
        <button className="link" style={{ float: "right" }} onClick={logout}>
          Sign out
        </button>
        <p style={{ textAlign: "center", marginTop: "2.5em" }}>
          <button onClick={storageDeposit}>storage deposit</button>
        </p>
      </main>
    );
  }

  //TODO: コンポーネントを分ける

  return (
    // use React Fragment, <>, to avoid wrapping elements in unnecessary divs
    <>
      <button className="link" style={{ float: "right" }} onClick={logout}>
        Sign out
      </button>
      <main>
        <h1>
          Hello
          {
            " " /* React trims whitespace around tags; insert literal space character when needed */
          }
          {window.accountId} !
        </h1>
        {console.log("before loop ", bikes)}
        {inProcess === true ? (
          <p> in process... </p>
        ) : (
          bikes.map((bike, index) => {
            return (
              <div style={{ display: "flex" }}>
                {console.log("in loop ", index, ":", bike)}
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
                  disabled={!bike.in_use && !bike.inspection}
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
        <form
          onSubmit={async (event) => {
            event.preventDefault();
            const { fieldset, account } = event.target.elements;
            const account_to_check = account.value;
            fieldset.disabled = true;
            try {
              await getThenSetBalance(account_to_check);
            } catch (e) {
              alert(e);
            }
            fieldset.disabled = false;
          }}
        >
          <fieldset id="fieldset">
            <label
              htmlFor="account"
              style={{
                display: "block",
                color: "var(--gray)",
                marginBottom: "0.5em",
              }}
            >
              type account to check balance
            </label>
            <div style={{ display: "flex" }}>
              <input autoComplete="off" id="account" style={{ flex: 1 }} />
              <button style={{ borderRadius: "0 5px 5px 0" }}>check</button>
            </div>
          </fieldset>
        </form>
        {accountToShowBalance && (
          <p>
            {accountToShowBalance}'s balance: {showBalance}
          </p>
        )}
        <form
          onSubmit={async (event) => {
            event.preventDefault();
            // get elements from the form using their id attribute
            const { fieldset, account } = event.target.elements;
            const account_to_transfer = account.value;
            fieldset.disabled = true;
            try {
              await ft_transfer(account_to_transfer);
            } catch (e) {
              alert(e);
            }
            fieldset.disabled = false;
          }}
        >
          <fieldset id="fieldset">
            <label
              htmlFor="account"
              style={{
                display: "block",
                color: "var(--gray)",
                marginBottom: "0.5em",
              }}
            >
              type account to transfer 30 FT
            </label>
            <div style={{ display: "flex" }}>
              <input autoComplete="off" id="account" style={{ flex: 1 }} />
              <button style={{ borderRadius: "0 5px 5px 0" }}>transfer</button>
            </div>
          </fieldset>
        </form>
      </main>
    </>
  );
}
