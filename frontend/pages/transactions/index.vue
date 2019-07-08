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
    processInput () {
      if (this.value === 'All') {
        this.tranasction = {}
        this.transactions = this.$store.state.transactions.transactions
      } else {
        this.typePage = 1
        this.transactions = {}
        this.loadmore()
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
  width: 40%;
  .multiselect__option--highlight {
    background: #14CCB7;
    outline: none;
    color: #fff;
  }
  .multiselect__option--highlight:after {
    content: attr(data-select);
    background: #14CCB7;
    color: #fff;
  }
  .multiselect__option--selected {
    background: #f3f3f3;
    color: #35495e;
    font-weight: 700;
  }
  .multiselect__option--selected:after {
    content: attr(data-selected);
    color: silver;
  }
  .multiselect__option--selected.multiselect__option--highlight {
    background: #FF0D6A;
    color: #fff;
  }
  .multiselect__option--selected.multiselect__option--highlight:after {
    background: #FF0D6A;
    content: attr(data-deselect);
    color: #fff;
  }
  .multiselect--disabled .multiselect__current,
  .multiselect--disabled .multiselect__select {
    background: #ededed;
    color: #a6a6a6;
  }
  .multiselect__option--disabled {
    background: #ededed !important;
    color: #a6a6a6 !important;
    cursor: text;
    pointer-events: none;
  }
}
</style>
