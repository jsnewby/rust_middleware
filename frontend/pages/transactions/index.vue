<template>
  <div class="app-transactions">
    <PageHeader title="Transactions">
      <BreadCrumbs />
    </PageHeader>
    <TxList>
      <nuxt-link
        v-for="(item, index) in Object.values(transactions).reverse()"
        :key="index"
        :to="`/transactions/${item.hash}`"
      >
        <TXListItem
          :data="item"
        />
      </nuxt-link>
    </TxList>
    <LoadMoreButton @update="loadmore" />
  </div>
</template>

<script>

import TxList from '../../partials/transactions/txList'
import TXListItem from '../../partials/transactions/txListItem'
import PageHeader from '../../components/PageHeader'
import BreadCrumbs from '../../components/breadCrumbs'
import LoadMoreButton from '../../components/loadMoreButton'

import { mapState } from 'vuex'

export default {
  name: 'AppTransactions',
  components: {
    TxList,
    TXListItem,
    PageHeader,
    BreadCrumbs,
    LoadMoreButton
  },
  data () {
    return {
      page: 1
    }
  },
  computed: {
    ...mapState('transactions', [
      'transactions'
    ])
  },
  methods: {
    loadmore () {
      this.$store.dispatch('transactions/getLatestTransactions', { 'page': this.page, 'numTransactions': 10 })
      this.page += 1
    }
  }
}
</script>
