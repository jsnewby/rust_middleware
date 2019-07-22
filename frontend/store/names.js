import Vue from 'vue'
import axios from 'axios'
export const state = () => ({
  names: {}
})

export const mutations = {
  setNames (state, names) {
    for (let name of names) {
      Vue.set(state.names, name.id, name)
    }
  }
}

export const actions = {
  getNames: async function ({ rootState: { nodeUrl }, commit }, { page, limit }) {
    try {
      const url = `${nodeUrl}/middleware/names?limit=${limit}&page=${page}`
      const names = await axios.get(url)
      console.info('MDW 🔗 ' + url)
      commit('setNames', names.data)
      return names.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  }
}
