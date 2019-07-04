<template>
  <div class="app-transactions">
    <PageHeader
      title="Account"
      :has-crumbs="true"
      :page="{to: `/account/transactions/${$route.params.id}`, name: `${account.id}(${amount})`}"
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
import prefixAmount from '../../../plugins/filters/prefixedAmount'

export default {
  name: 'AccountTransactions',
  components: {
    TxList,
    TXListItem,
    PageHeader,
    LoadMoreButton
  },
  filters: {
    prefixAmount
  },
  data () {
    return {
      account: {
        balance: 0
      },
      transactions: [],
      page: 1
    }
  },
  computed: {
    amount () {
      return prefixAmount(this.account.balance)
    }
  },
  async asyncData ({ store, params }) {
    const transactions = await store.dispatch('transactions/getTransactionByAccount', { account: params.id, page: 1, limit: 10 })
    const account = await store.dispatch('account/getAccountDetails', params.id)
    return { address: params.id, transactions, page: 2, account }
  },
  methods: {
    async loadMore () {
      const transactions = await this.$store.dispatch('transactions/getTransactionByAccount', { account: this.account.id, page: this.page, limit: 10 })
      this.transactions = [...this.transactions, ...transactions]
      this.page += 1
    }
  }
}
</script>
