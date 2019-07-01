<template>
  <div class="app-transactions">
    <PageHeader
      title="Contracts Transactions"
      :has-crumbs="true"
      :page="{to: '/Contracts', name: 'Contracts'}"
      :subpage="{to: `/contracts/transactions/${$route.params.id}`, name: 'Contract Transactions'}"
    />
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

export default {
  name: 'ChannelTransactions',
  components: {
    TxList,
    TXListItem,
    PageHeader
  },
  data () {
    return {
      contract: '',
      transactions: []
    }
  },
  async asyncData ({ store, params }) {
    const transactions = await store.dispatch('contracts/getContractTx', params.id)
    return { contract: params.id, transactions }
  }
}
</script>
