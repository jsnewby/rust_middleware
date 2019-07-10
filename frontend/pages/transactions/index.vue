<template>
  <div class="app-transactions">
    <PageHeader
      title="Transactions"
      :has-crumbs="true"
      :page="{to: '/transactions', name: 'Transactions'}"
    />
    <div class="filter">
      <multiselect
        v-model="value"
        :options="options"
        :allow-empty="false"
        :loading="loading"
        placeholder="Select transaction type...."
        @input="processInput"
      />
    </div>
    <div v-if="Object.keys(transactions).length > 0">
      <TxList>
        <TXListItem
          v-for="(item, index) in Object.values(transactions)"
          :key="index"
          :data="item"
        />
      </TxList>
      <LoadMoreButton @update="loadmore" />
    </div>
    <div v-if="loading">
      Loading....
    </div>
    <div v-if="!loading && Object.keys(transactions).length == 0">
      No matching transactions found for the selected type.
    </div>
  </div>
</template>

<script>
import TxList from '../../partials/transactions/txList'
import TXListItem from '../../partials/transactions/txListItem'
import PageHeader from '../../components/PageHeader'
import LoadMoreButton from '../../components/loadMoreButton'
import Multiselect from 'vue-multiselect'

export default {
  name: 'AppTransactions',
  components: {
    TxList,
    TXListItem,
    PageHeader,
    LoadMoreButton,
    Multiselect
  },
  data () {
    return {
      typePage: 1,
      loading: false,
      value: 'All',
      transactions: this.$store.state.transactions.transactions,
      options: this.$store.state.filterOptions
    }
  },
  methods: {
    async loadmore () {
      if (this.value === 'All') {
        await this.getAllTx()
      } else {
        await this.getTxByType()
      }
    },
    async getAllTx () {
      const tx = await this.$store.dispatch(
        'transactions/getLatestTransactions',
        { limit: 10 }
      )
      tx.forEach(element => {
        this.transactions = { ...this.transactions, [element.hash]: element }
      })
    },
    async getTxByType () {
      const tx = await this.$store.dispatch('transactions/getTxByType', {
        page: this.typePage,
        limit: 10,
        txtype: this.value
      })
      tx.forEach(element => {
        this.transactions = { ...this.transactions, [element.hash]: element }
      })
      this.typePage += 1
    },
    async processInput () {
      if (this.value === 'All') {
        this.tranasction = {}
        this.transactions = this.$store.state.transactions.transactions
      } else {
        this.loading = true
        this.typePage = 1
        this.transactions = {}
        await this.loadmore()
        this.loading = false
      }
    }
  }
}
</script>
