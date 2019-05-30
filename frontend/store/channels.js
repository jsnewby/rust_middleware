import axios from 'axios'

export const state = () => ({
  channels: []
})

export const mutations = {
  setChannels (state, channels) {
    state.channels = channels
  }
}

export const actions = {
  getChannels: async function ({ rootState: { nodeUrl }, commit }) {
    try {
      const channels = await axios.get(nodeUrl + '/middleware/channels/active')
      commit('setChannels', channels.data)
      return channels.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  }
}
