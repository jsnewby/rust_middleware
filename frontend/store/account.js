import axios from 'axios'

export const actions = {
  getAccountDetails: async function ({ rootState: { nodeUrl }, commit }, account) {
    try {
      const acc = await axios.get(`${nodeUrl}/v2/accounts/${account}`)
      return acc.data
    } catch (e) {
      commit('catchError', 'Error', { root: true })
      const basicError = {
        id: account,
        balance: 0,
        error: 'Unable to fetch account details'
      }
      if (e.response.status === 500) {
        basicError.error = 'Account not found'
      }
      return basicError
    }
  }
}
