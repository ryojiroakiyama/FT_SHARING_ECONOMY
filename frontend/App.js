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
  storage_unregister,
  amount_to_use_bike,
} from "./assets/js/near/utils";

export default function App() {
  const [amountToUseBike, setAmountToUseBike] = useState(0);

  const [allBikeInfo, setAllBikeInfo] = useState([]);

  const [toShowBalance, setToShowBalance] = useState(false);
  const [balanceInfo, setBalanceInfo] = useState({});

  const RenderingStates = {
    SIGN_IN: "sign_in",
    REGISTRY: "registry",
    HOME: "home",
    TRANSACTION: "transaction",
  };
  const [renderingState, setRenderingState] = useState(RenderingStates.HOME);

  const bikeImg = require("./assets/img/bike.png");

  /**
   * bikeInfoオブジェクトを定義します.
   * allBikeInfoはbikeInfoオブジェクトの配列となります.
   * 各属性はログインアカウントと連携した情報になります.
   * available:  ログインアカウントはバイクを使用可能か否か
   * in_use:     同じく使用中か否か
   * inspection: 同じく点検中か否か
   */
  const initialBikeInfo = async () => {
    return { available: false, in_use: false, inspection: false };
  };

  const initialBalanceInfo = async () => {
    return { account_id: "", balance: 0 };
  };

  const createBikeInfo = async (index) => {
    let bike = await initialBikeInfo();
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

  const createAllBikeInfo = async () => {
    const num = await num_of_bikes();
    console.log("Num of bikes:", num);

    let new_bikes = [];
    for (let i = 0; i < num; i++) {
      const bike = await createBikeInfo(i);
      new_bikes.push(bike);
    }

    setAllBikeInfo(new_bikes);
    console.log("Set bikes: ", new_bikes);
  };

  const updateBikeInfo = async (index) => {
    const new_bike = await createBikeInfo(index);

    allBikeInfo[index] = new_bike;
    setAllBikeInfo(allBikeInfo);
    console.log("Update bikes: ", allBikeInfo);
  };

  /**
   * idで指定されたユーザがftコントラクトに対して登録(Storage Registration)を済ましているかを確認します.
   */
  const isRegistered = async (account_id) => {
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
  // サイン後にもブラウザのページがリロードされるので, この内容が実行されます.
  useEffect(() => {
    const getAmountToUseBike = async () => {
      const amount = await amount_to_use_bike();
      setAmountToUseBike(BigInt(amount));
    };

    const checkUserRegistry = async (account_id) => {
      const is_registered = await isRegistered(account_id);
      if (!is_registered) {
        setRenderingState(RenderingStates.REGISTRY);
      }
    };

    createAllBikeInfo();
    getAmountToUseBike();

    // renderingStateを設定します.
    // ユーザがサインインを済ませていなければSIGN_INをセットします.
    // 済ませていれば登録を確認します.
    if (!window.walletConnection.isSignedIn()) {
      setRenderingState(RenderingStates.SIGN_IN);
    } else {
      checkUserRegistry(window.accountId);
    }
  }, []);

  /**
   * ユーザが自信を登録.完了したらbikeコントラクトからユーザへftを送る
   */
  const registerThenTransferFt = async () => {
    try {
      await storage_deposit().then(async (value) => {
        console.log("Result of storage_deposit: ", value);
        await transfer_ft_to_new_user(window.accountId);
      });
    } catch (e) {
      alert(e);
    }
  };

  /**
   * ft_transfer_callの実行.
   * bikeコントラクトにft送金+使用するバイクをindexで指定 => bikeコントラクト側で指定バイクの使用処理が実行されます.
   * トランザクションへのサイン後は画面がリロードされます.
   */
  const transferFtToUseBike = async (index) => {
    console.log("Transfer ft to use bike");

    // 不要なトランザクションを避けるためにユーザの残高を確認
    const balance = await ft_balance_of(window.accountId);
    if (balance < amountToUseBike) {
      alert(amountToUseBike + "ft is required to use the bike");
    } else {
      try {
        ft_transfer_call(index, amountToUseBike.toString());
      } catch (e) {
        alert(e);
      }
    }
  };

  /**
   * バイクを点検, allBikeInfoをアップデートします.
   */
  const inspectBikeThenUpdateInfo = async (index) => {
    console.log("Inspect bike");
    setRenderingState(RenderingStates.TRANSACTION);

    try {
      await inspect_bike(index);
    } catch (e) {
      alert(e);
    }
    await updateBikeInfo(index);

    setRenderingState(RenderingStates.HOME);
  };

  /**
   * バイクを返却, allBikeInfoをアップデートします.
   */
  const returnBikeThenUpdateInfo = async (index) => {
    console.log("Return bike");
    setRenderingState(RenderingStates.TRANSACTION);

    try {
      await return_bike(index);
    } catch (e) {
      alert(e);
    }
    await updateBikeInfo(index);

    setRenderingState(RenderingStates.HOME);
  };

  /**
   * 指定されたaccount_idの残高を取得し, 情報をbalanceInfoにセットします.
   */
  const getBalanceThenSet = async (account_id) => {
    let balance_info = await initialBalanceInfo();
    const balance = await ft_balance_of(account_id);
    balance_info.account_id = account_id;
    balance_info.balance = balance;
    setBalanceInfo(balance_info);
    setToShowBalance(true);
  };

  const signOutButton = () => {
    return (
      <button className="link" style={{ float: "right" }} onClick={logout}>
        Sign out
      </button>
    );
  };

  const unregisterButton = () => {
    return (
      <button
        className="link"
        style={{ float: "right" }}
        onClick={storage_unregister}
      >
        Unregister
      </button>
    );
  };

  const requireSignIn = () => {
    return (
      <div>
        <main>
          <p style={{ textAlign: "center", marginTop: "2.5em" }}>
            <button onClick={login}>Sign in</button>
          </p>
        </main>
      </div>
    );
  };

  const requireRegistry = () => {
    return (
      <div>
        {signOutButton()}
        <div style={{ textAlign: "center" }}>
          <h5>
            Storage registration in ft contract is required before using the
            bike app
          </h5>
        </div>
        <main>
          <p style={{ textAlign: "center", marginTop: "2.5em" }}>
            <button onClick={registerThenTransferFt}>storage deposit</button>
          </p>
        </main>
      </div>
    );
  };

  const header = () => {
    return <h1>Hello {window.accountId} !</h1>;
  };

  const transaction = () => {
    return (
      <div>
        {header()}
        <main>
          <p> in process... </p>
        </main>
      </div>
    );
  };

  const bikeContents = () => {
    return (
      <div>
        {allBikeInfo.map((bike, index) => {
          return (
            <div class="bike" style={{ display: "flex" }}>
              <div class="bike_img">
                <img src={bikeImg} />
              </div>
              <div class="bike_index">: {index}</div>
              <button
                disabled={!bike.available}
                onClick={() => transferFtToUseBike(index)}
              >
                use
              </button>
              <button
                disabled={!bike.available}
                onClick={() => inspectBikeThenUpdateInfo(index)}
              >
                inspect
              </button>
              <button
                disabled={!bike.in_use && !bike.inspection}
                onClick={() => returnBikeThenUpdateInfo(index)}
              >
                return
              </button>
            </div>
          );
        })}
      </div>
    );
  };

  const checkBalance = () => {
    return (
      <div class="balance_content">
        <button onClick={() => getBalanceThenSet(window.accountId)}>
          check my balance
        </button>
        <button
          style={{ marginTop: "0.1em" }}
          onClick={() => getBalanceThenSet(window.bikeContract.contractId)}
        >
          check contract's balance
        </button>
        <span>or</span>
        <form
          onSubmit={async (event) => {
            event.preventDefault();
            const { fieldset, account } = event.target.elements;
            const account_to_check = account.value;
            fieldset.disabled = true;
            try {
              await getBalanceThenSet(account_to_check);
            } catch (e) {
              alert(e);
            }
            fieldset.disabled = false;
          }}
        >
          <fieldset id="fieldset">
            <div style={{ display: "flex" }}>
              <input autoComplete="off" id="account" placeholder="account id" />
              <button style={{ borderRadius: "0 5px 5px 0" }}>check</button>
            </div>
          </fieldset>
        </form>
        {toShowBalance && (
          <div>
            <p>{balanceInfo.account_id}'s</p>
            <p>balance: {balanceInfo.balance}</p>
          </div>
        )}
      </div>
    );
  };

  const transferFt = () => {
    return (
      <div>
        <form
          onSubmit={async (event) => {
            event.preventDefault();
            const { fieldset, account } = event.target.elements;
            const account_to_transfer = account.value;
            fieldset.disabled = true;
            try {
              await ft_transfer(
                account_to_transfer,
                amountToUseBike.toString()
              );
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
                marginTop: "1em",
              }}
            >
              give someone {amountToUseBike.toString()} ft
            </label>
            <div style={{ display: "flex" }}>
              <input
                autoComplete="off"
                id="account"
                style={{ flex: 1 }}
                placeholder="account id"
              />
              <button style={{ borderRadius: "0 5px 5px 0" }}>transfer</button>
            </div>
          </fieldset>
        </form>
      </div>
    );
  };

  const home = () => {
    return (
      <div>
        {signOutButton()}
        {unregisterButton()}
        {header()}
        <main>
          {bikeContents()}
          {checkBalance()}
          {transferFt()}
        </main>
      </div>
    );
  };

  switch (renderingState) {
    case RenderingStates.SIGN_IN:
      return <div>{requireSignIn()}</div>;

    case RenderingStates.REGISTRY:
      return <div>{requireRegistry()}</div>;

    case RenderingStates.TRANSACTION:
      return <div>{transaction()}</div>;

    case RenderingStates.HOME:
      return <div>{home()}</div>;
  }
}
