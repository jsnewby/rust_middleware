<template>
  <div class="app-contracts">
    <PageHeader
      title="Contracts"
      :has-crumbs="true"
      :page="{to: '/contracts', name: 'Contracts'}"
    />
    <ContractList>
      <Contract
        v-for="(item, index) in Object.values(contracts)"
        :key="index"
        :data="item"
      />
    </ContractList>
    <LoadMoreButton @update="loadMore" />
  </div>
</template>

<script>

import ContractList from '../../partials/contractList'
import Contract from '../../partials/contract'
import PageHeader from '../../components/PageHeader'
import LoadMoreButton from '../../components/loadMoreButton'
import { mapState } from 'vuex'

export default {
  name: 'AppContracts',
  components: {
    ContractList,
    Contract,
    PageHeader,
    LoadMoreButton
  },
  data () {
    return {
      page: 1
    }
  },
  computed: {
    ...mapState('contracts', [
      'contracts'
    ])
  },
  beforeMount () {
    this.loadMore()
  },
  methods: {
    loadMore () {
      this.$store.dispatch('contracts/getContracts', { 'page': this.page, 'limit': 10 })
      this.page += 1
    }
  }
}
</script>
