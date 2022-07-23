import { connect, Contract, keyStores, WalletConnection } from 'near-api-js'
import getConfig from './config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')

// Initialize contract & set global variables
export async function initContract() {
  // Initialize connection to the NEAR testnet
  const near = await connect(Object.assign({ deps: { keyStore: new keyStores.BrowserLocalStorageKeyStore() } }, nearConfig))

  // Initializing Wallet based Account. It can work with NEAR testnet wallet that
  // is hosted at https://wallet.testnet.near.org
  window.walletConnection = new WalletConnection(near)
  console.log("walletConnection:", window.walletConnection)
  console.log("window.walletConnection.account()", window.walletConnection.account())

  // Getting the Account ID. If still unauthorized, it's just empty string
  window.accountId = window.walletConnection.getAccountId()

  // Initializing our contract APIs by contract name and configuration
  window.bikeContract = await new Contract(window.walletConnection.account(), nearConfig.bikeContractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: ['get_greeting', 'get_bikes'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: ['set_greeting', 'use_bike', 'return_bike', 'inspect_bike'],
  })

  // Initializing our contract APIs by contract name and configuration
  window.ftContract = await new Contract(window.walletConnection.account(), nearConfig.ftContractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: ['ft_balance_of', 'storage_balance_of'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: ['storage_deposit', 'ft_transfer'],
  })
}

export function logout() {
  window.walletConnection.signOut()
  // reload page
  window.location.replace(window.location.origin + window.location.pathname)
}

export function login() {
  // Allow the current app to make calls to the specified contract on the
  // user's behalf.
  // This works by creating a new access key for the user's account and storing
  // the private key in localStorage.
  window.walletConnection.requestSignIn(nearConfig.bikeContractName)
}

export async function set_greeting(message){
  let response = await window.bikeContract.set_greeting({
    args:{message: message}
  })
  return response
}

export async function use_bike(index){
  let response = await window.bikeContract.use_bike({
    args:{index: index}
  })
  return response
}

export async function return_bike(index){
  let response = await window.bikeContract.return_bike({
    args:{index: index}
  })
  return response
}

export async function inspect_bike(index){
  let response = await window.bikeContract.inspect_bike({
    args:{index: index}
  })
  return response
}

export async function get_greeting(){
  let greeting = await window.bikeContract.get_greeting()
  return greeting
}

export async function get_bikes(){
  let bikes = await window.bikeContract.get_bikes()
  return bikes
}

export async function ft_balance_of(account_id){
  let balance = await window.ftContract.ft_balance_of({
    account_id: account_id
  })
  return balance
}

export async function storage_balance_of(account_id){
  let balance = await window.ftContract.storage_balance_of({
    account_id: account_id
  })
  return balance
}

// 引数を省略してユーザにstorage_depositを負わせる
// ガス代テキトー
export async function storage_deposit(){
  let response = await window.ftContract.storage_deposit({}, "300000000000000", "1250000000000000000000")
  return response
}

// とりあえず引数固定, 省略
export async function ft_transfer(){
  let response = await window.ftContract.ft_transfer({
    args:{index: nearConfig.bikeContractName, amount: "19"}
  })
  return response
}
