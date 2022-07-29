import "regenerator-runtime/runtime";
import React, { useEffect, useState } from "react";

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
  transfer_ft_to_new_user,
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

  // bikesの各要素のフィールドと各属性の初期値を定義します.
  // 各属性はログインアカウントと連携した情報になります.
  // available:  ログインアカウントはバイクを使用可能か否か
  // in_use:     同じく使用中か否か
  // inspection: 同じく点検中か否か
  const bikeField = async () => {
    return { available: false, in_use: false, inspection: false };
  };

  // 特定のバイクに関する情報を取得, オブジェクトにセットして返却します.
  const getSpecificBike = async (index) => {
    let bike = await bikeField();
    await is_available(index).then((is_available) => {
      if (is_available) {
        bike.available = is_available;
        return bike;
      }
    });
    await who_is_using(index).then((user_id) => {
      if (window.accountId === user_id) {
        bike.in_use = true;
        return bike;
      }
    });
    await who_is_inspecting(index).then((inspector_id) => {
      if (window.accountId === inspector_id) {
        bike.inspection = true;
      }
    });
    return bike;
  };

  // 全てのバイクの情報を取得し, bikesを作成します.
  const setAllBikes = async () => {
    const num = await num_of_bikes();
    console.log("Num of bikes:", num);
    let new_bikes = [];
    for (let i = 0; i < num; i++) {
      const bike = await getSpecificBike(i);
      new_bikes.push(bike);
    }
    setBikes(new_bikes);
    console.log("Set bikes: ", new_bikes);
  };

  // 特定のバイクの情報をアップデートします.
  const updateBikes = async (index) => {
    const new_bike = await getSpecificBike(index);
    bikes[index] = new_bike;
    setBikes(bikes);
    console.log("Update bikes: ", bikes);
  };

  // ユーザがFTコントラクトにストレージステーキングしているかを確認します.
  const isStorageRegistered = async (account_id) => {
    const balance = await storage_balance_of(account_id);
    console.log("user's storage balance: ", balance);
    // ストレージ残高にnullが返ってくる場合は未登録を意味します.
    if (balance === null) {
      console.log("account is not yet registered");
      return false;
    } else {
      return true;
    }
  };

  // 初回レンダリング時の処理.
  // サイン後はページがリロードされるので,サインをする度に初回レンダリングで実行されます.
  useEffect(() => {
    // 全てのバイクの情報を取得, bikesにセットします
    setAllBikes();

    // ユーザのアカウントがFTコントラクトに登録されているかを確認します.
    const checkStorageRegistered = async (account_id) => {
      const is_registered = await isStorageRegistered(account_id);
      setStorageRegistered(is_registered);
    };
    // 空文字列(ユーザがサインイン前)はエラーを引き起こすので条件式を使います
    if (window.accountId) {
      checkStorageRegistered(window.accountId);
    }
  }, []);

  // ストレージを登録し, bikeコントラクトからユーザに30FTを送信します
  const storageDeposit = async () => {
    try {
      storage_deposit().then((value) => {
        console.log("Result of storage_deposit: ", value);
      });
      await transfer_ft_to_new_user(window.accountId);
    } catch (e) {
      alert(e);
    }
  };

  // ft_trasnfer_callを呼ぶことでBikeコントラクトにFT送金+使用するバイクをindexで指定
  // => Bikeコントラクト側で指定バイクの使用処理が実行されます.
  // トランザクションへのサイン後は画面がリロードされます.
  const trasferFtToUseBike = async (index) => {
    console.log("Trasfer FT to use bike");
    // 余分なトランザクションを避けるためにユーザの残高を確認
    const balance = await ft_balance_of(window.accountId);
    if (balance < 30) {
      alert("30 FT is required to use the bike");
    } else {
      try {
        ft_transfer_call(index);
      } catch (e) {
        alert(e);
      }
    }
  };

  // バイクを点検, 情報をアップデート
  const inspectThenUpdateBikes = async (index) => {
    console.log("Inspect bike");
    setInProcess(true);
    try {
      await inspect_bike(index);
    } catch (e) {
      alert(e);
    }
    await updateBikes(index);
    setInProcess(false);
  };

  // バイクを返却, 情報をアップデート
  const returnThenUpdateBikes = async (index) => {
    console.log("Return bike");
    setInProcess(true);
    try {
      await return_bike(index);
    } catch (e) {
      alert(e);
    }
    await updateBikes(index);
    setInProcess(false);
  };

  //TODO: 二つ変数使っているのが気になる。。
  const getThenSetBalance = async (account_id) => {
    const user_balance = await ft_balance_of(account_id);
    setShowBalance(user_balance);
    setAccountToShowBalance(account_id);
  };

  // サインインしていなければサインイン画面を返却
  if (!window.walletConnection.isSignedIn()) {
    return (
      <main>
        <p style={{ textAlign: "center", marginTop: "2.5em" }}>
          <button onClick={login}>Sign in</button>
        </p>
      </main>
    );
  }

  // ストレージレジスト画面を返却
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
                  onClick={() => inspectThenUpdateBikes(index)}
                  style={{ borderRadius: "5px 5px 5px 5px" }}
                >
                  inspect
                </button>
                <button
                  disabled={!bike.in_use && !bike.inspection}
                  onClick={() => returnThenUpdateBikes(index)}
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
