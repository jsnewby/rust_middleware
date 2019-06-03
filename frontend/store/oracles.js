import Vue from 'vue'
import axios from 'axios'

export const state = () => ({
  oracles: {}
})

export const mutations = {
  setOracles (state, oracles) {
    for (let oracle of oracles) {
      Vue.set(state.oracles, oracle.transaction_hash, oracle)
    }
  }
}

export const actions = {
  getOracles: async function ({ rootState: { nodeUrl }, commit }, { page, limit }) {
    try {
      const oracles = await axios.get(nodeUrl + '/middleware/oracles/list?limit=' + limit + '&page=' + page)
      commit('setOracles', oracles.data)
      return oracles.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  }
}
