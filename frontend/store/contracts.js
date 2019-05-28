import axios from 'axios'

export const state = () => ({
  contracts: []
})

export const mutations = {
  setContracts (state, contracts) {
    state.contracts = contracts
  }
}

export const actions = {
  getAllContracts: async function ({ rootState: { nodeUrl }, commit }) {
    try {
      const contracts = await axios.get(nodeUrl + '/middleware/contracts/all')
      commit('setContracts', contracts.data)
      return contracts.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  }
}
