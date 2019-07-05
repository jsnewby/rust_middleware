<template>
  <div class="app-transactions">
    <PageHeader
      title=" Transactions"
      :has-crumbs="true"
      :page="{to: '/transactions', name: 'Transactions'}"
    />
    <multiselect
      v-model="value"
      :options="options"
      placeholder="Transaction Type"
    />
    <TxList>
      <TXListItem
        v-for="(item, index) in Object.values(transactions).reverse()"
        :key="index"
        :data="item"
      />
    </TxList>
    <LoadMoreButton @update="loadmore" />
  </div>
</template>

<script>

import TxList from '../../partials/transactions/txList'
import TXListItem from '../../partials/transactions/txListItem'
import PageHeader from '../../components/PageHeader'
import LoadMoreButton from '../../components/loadMoreButton'
import Multiselect from 'vue-multiselect'
import { mapState } from 'vuex'

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
      page: 1,
      value: 'All',
      options: ['SpendTx', 'OracleRegisterTx', 'OracleExtendTx', 'OracleQueryTx', 'OracleRespondTx', 'NamePreclaimTx', 'NameClaimTx', 'NameUpdateTx', 'NameTransferTx', 'NameRevokeTx', 'ChannelCreateTx', 'ContractCallTx', 'ContractCreateTx', 'ChannelDepositTx', 'ChannelWithdrawTx', 'ChannelCloseMutualTx', 'ChannelForceProgressTx', 'ChannelCloseSoloTx', 'ChannelSlashTx', 'ChannelSettleTx', 'ChannelSnapshotSoloTx']
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
