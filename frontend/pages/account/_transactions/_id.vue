<template>
  <div class="app-transactions">
    <PageHeader
      title="Account"
      :has-crumbs="true"
      :page="{to: `/account/transactions/${$route.params.id}`, name: `${$route.params.id}`}"
    />
    <div v-if="transactions.length > 0">
      <TxList>
        <TXListItem
          v-for="tx of transactions"
          :key="tx.hash"
          :data="tx"
          :address="`${$route.params.id}`"
        />
      </TxList>
      <LoadMoreButton @update="loadMore" />
    </div>
    <div v-else>
      Nothing to see here right now...
    </div>
  </div>
</template>

<script>

import TxList from '../../../partials/transactions/txList'
import TXListItem from '../../../partials/transactions/txListItem'
import PageHeader from '../../../components/PageHeader'
import LoadMoreButton from '../../../components/loadMoreButton'

export default {
  name: 'ChannelTransactions',
  components: {
    TxList,
    TXListItem,
    PageHeader,
    LoadMoreButton
  },
  data () {
    return {
      address: '',
      transactions: [],
      page: 1
    }
  },
  async asyncData ({ store, params }) {
    const transactions = await store.dispatch('transactions/getTransactionByAccount', { account: params.id, page: 1, limit: 10 })
    return { address: params.id, transactions, page: 2 }
  },
  methods: {
    async loadMore () {
      const transactions = await this.$store.dispatch('transactions/getTransactionByAccount', { account: this.address, page: this.page, limit: 10 })
      this.transactions = [...this.transactions, ...transactions]
      this.page += 1
    }
  }
}
</script>
