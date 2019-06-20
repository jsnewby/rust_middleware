<template>
  <div class="app-transactions">
    <PageHeader title="Channel Transactions">
      <BreadCrumbs />
    </PageHeader>
    <TxList>
      <TXListItem
        v-for="tx of transactions"
        :key="tx.hash"
        :data="tx"
      />
    </TxList>
  </div>
</template>

<script>

import TxList from '../../../partials/transactions/txList'
import TXListItem from '../../../partials/transactions/txListItem'
import PageHeader from '../../../components/PageHeader'
import BreadCrumbs from '../../../components/breadCrumbs'

export default {
  name: 'ChannelTransactions',
  components: {
    TxList,
    TXListItem,
    PageHeader,
    BreadCrumbs
  },
  data () {
    return {
      transactions: []
    }
  },
  async asyncData ({ store, params }) {
    const transactions = await store.dispatch('channels/getChannelTx', params.id)
    return { transactions }
  }
}
</script>
