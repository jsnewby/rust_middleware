import Vue from 'vue'
import axios from 'axios'

export const state = () => ({
  generations: {},
  hashToHeight: {},
  lastFetchedGen: 0
})

export const mutations = {
  setGenerations (state, generations) {
    for (let i of Object.keys(generations)) {
      const generation = generations[i]
      if (!state.generations.hasOwnProperty(generation.height)) {
        Vue.set(state.hashToHeight, generation.hash, generation.height)
        Vue.set(state.generations, generation.height, generation)
      }
    }
  },
  setLastFetched (state, last) {
    state.lastFetchedGen = last
  }
}

export const actions = {
  getLatestGenerations: async function ({ state, rootState: { nodeUrl, height }, commit, dispatch }, maxBlocks) {
    try {
      const { start, end } = calculateBlocksToFetch(height, state.lastFetchedGen, maxBlocks)
      const generations = await axios.get(nodeUrl + 'middleware/generations/' + start + '/' + end)
      commit('setGenerations', generations.data.data)
      commit('setLastFetched', start)
      return generations.data.data
    } catch (e) {
      console.log(e)
      commit('catchError', 'Error', { root: true })
    }
  },
  nuxtServerInit ({ dispatch }, context) {
    return (
      dispatch('getLatestGenerations', 10)
    )
  },
  updateMicroBlocks: async function ({ state, commit, dispatch }, microBlock) {

  },
  updateTransactions: async function ({ state, commit, dispatch }, microBlock) {

  }
}

function calculateBlocksToFetch (height, lastFetchedGen, maxBlocks) {
  let start = 0
  let end = 0
  if (!lastFetchedGen) {
    start = height - maxBlocks
    end = height
  } else {
    start = lastFetchedGen - maxBlocks - 1
    end = lastFetchedGen - 1
  }
  return { start, end }
}
