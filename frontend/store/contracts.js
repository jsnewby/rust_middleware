import Vue from 'vue'
import axios from 'axios'

export const state = () => ({
  contracts: []
})

export const mutations = {
  setContracts (state, contracts) {
    for (let contract of contracts) {
      Vue.set(state.contracts, contract.contract_id, contract)
    }
  }
}

export const actions = {
  getContracts: async function ({ rootState: { nodeUrl }, commit }, { page, limit }) {
    try {
      const contracts = await axios.get(nodeUrl + '/middleware/contracts/all?limit=' + limit + '&page=' + page)
      commit('setContracts', contracts.data)
      return contracts.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  }
}
