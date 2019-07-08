import Vue from 'vue'
import axios from 'axios'

export const state = () => ({
  transactions: {},
  lastPage: 0
})

export const mutations = {
  setTransactions (state, transactions) {
    for (let i = 0; i < transactions.length; i++) {
      const transaction = transactions[i]
      if (!state.transactions.hasOwnProperty(transaction.hash)) {
        Vue.set(state.transactions, transaction.hash, transaction)
      }
    }
  },
  setLastPage (state, page) {
    state.lastPage = page
  }
}

export const actions = {
  getLatestTransactions: async function ({ state, rootState: { nodeUrl, height }, commit }, { limit }) {
    try {
      const page = state.lastPage + 1
      const transactions = await axios.get(`${nodeUrl}/middleware/transactions/interval/1/${height}?limit=${limit}&page=${page}`)
      commit('setTransactions', transactions.data.transactions)
      commit('setLastPage', page)
      return transactions.data.transactions
    } catch (e) {
      commit('catchError', 'Error', { root: true })
    }
  },
  getTxByType: async function ({ rootState: { nodeUrl, height }, commit }, { page, limit, txtype }) {
    try {
      const transactions = await axios.get(`${nodeUrl}/middleware/transactions/interval/1/${height}?txtype=${txtype}&limit=${limit}&page=${page}`)
      return transactions.data.transactions
    } catch (e) {
      commit('catchError', 'Error', { root: true })
    }
  },
  getTransactionByHash: async function ({ rootState: { nodeUrl }, commit }, hash) {
    try {
      const tx = await axios.get(nodeUrl + '/v2/transactions/' + hash)
      commit('setTransactions', [tx.data])
      return tx.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  },
  getTransactionByAccount: async function ({ rootState: { nodeUrl }, commit }, { account, limit, page }) {
    try {
      const tx = await axios.get(`${nodeUrl}/middleware/transactions/account/${account}?page=${page}&limit=${limit}`)
      return tx.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  },
  nuxtServerInit ({ dispatch }, context) {
    return (
      dispatch('getLatestTransactions', { limit: 10 })
    )
  }
}
