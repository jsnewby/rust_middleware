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
      prev: 'All',
      transactions: this.$store.state.transactions.transactions,
      options: [
        'All',
        'SpendTx',
        'OracleRegisterTx',
        'OracleExtendTx',
        'OracleQueryTx',
        'OracleResponseTx',
        'NamePreclaimTx',
        'NameClaimTx',
        'NameUpdateTx',
        'NameTransferTx',
        'NameRevokeTx',
        'GAAttachTx',
        'ContractCallTx',
        'ContractCreateTx',
        'ChannelCreateTx',
        'ChannelDepositTx',
        'ChannelWithdrawTx',
        'ChannelCloseMutualTx',
        'ChannelForceProgressTx',
        'ChannelCloseSoloTx',
        'ChannelSlashTx',
        'ChannelSettleTx',
        'ChannelSnapshotSoloTx'
      ]
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
<style lang="scss">
.filter {
  display: flex;
  flex-direction: column;
  padding: 0.6rem 0.6rem 0 0;
  border-radius: 0.4rem;
  margin-bottom: 1rem;

  @media(min-width: 360px) {
    width: 80%;
  }

  @media(min-width: 768px) {
    width: 40%;
  }

  .multiselect__option--highlight {
    background: #14CCB7;
  }
  .multiselect__option--highlight:after {
    background: #14CCB7;
  }
  .multiselect__option--selected.multiselect__option--highlight {
    background: #FF0D6A;
  }
  .multiselect__option--selected.multiselect__option--highlight:after {
    background: #FF0D6A;
  }
  .multiselect__spinner:after,.multiselect__spinner:before {
    border-top-color:#FF0D6A;
  }
}
</style>
