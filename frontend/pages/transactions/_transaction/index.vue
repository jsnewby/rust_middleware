<template>
  <div class="app-transaction">
    <h1>Transaction Overview</h1>
    <GenerationDetails
      :data="generation"
      :dynamic-data="height"
      :status="loading"
    />
    <TransactionDetails
      :status="loading"
      :data="transaction"
    />
  </div>
</template>

<script>
import GenerationDetails from '../../../partials/generationDetails'
import TransactionDetails from '../../../partials/transactionDetails'

export default {
  name: 'AppTransaction',
  components: {
    GenerationDetails,
    TransactionDetails
  },
  data () {
    return {
      transation: {},
      generation: {},
      height: 0,
      loading: true
    }
  },
  async asyncData ({ store, params: { transaction } }) {
    let txDetails = null
    let generation = null
    let height = null
    if (store.transactions) {
      txDetails = store.transactions.transactions[txDetails]
    }
    if (!txDetails) {
      txDetails = await store.dispatch('transactions/getTransactionByHash', transaction)
    }
    if (store.generations) {
      generation = store.generations.generations[txDetails.block_height]
    }
    if (!generation) {
      generation = (await store.dispatch('generations/getGenerationByRange', { start: txDetails.block_height, end: txDetails.block_height }))[txDetails.block_height]
    }
    if (!store.height) {
      height = await store.dispatch('height')
    }
    return { transaction: txDetails, generation, height, loading: false }
  }
}
</script>

<style scoped>

</style>
